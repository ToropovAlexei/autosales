import asyncio
import logging
import os
import re
import subprocess
import time
import json
from pathlib import Path
from collections import deque
from urllib.parse import urlencode

import requests
from dotenv import load_dotenv
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry
from telethon.sync import TelegramClient
from telethon.errors import SessionPasswordNeededError
from aiohttp import web
import redis.asyncio as redis

from logging_config import setup_logging

# --- Configuration ---
load_dotenv()

# For BotFather interaction
API_ID = os.getenv("API_ID")
API_HASH = os.getenv("API_HASH")
SESSION_NAME = "bot_creator"

# For Backend API
API_URL = f'{os.getenv("API_URL")}'
SERVICE_TOKEN = os.getenv("SERVICE_TOKEN")

# For Redis
REDIS_HOST = os.getenv("REDIS_HOST", "localhost")
REDIS_PORT = int(os.getenv("REDIS_PORT", 6379))

# For Webhook Server
WEBHOOK_HOST = os.getenv("WEBHOOK_HOST", "0.0.0.0")
WEBHOOK_PORT = int(os.getenv("WEBHOOK_PORT", 8080))

# General Settings
TOKENS_FILE = Path("tokens.txt")
UNAVAILABLE_TOKENS_FILE = Path("unavailable_tokens.txt")
BOT_COMMAND = ["python", "main.py"]
HEALTH_CHECK_INTERVAL = 60
STARTUP_WAIT_TIME = 10
# --- End Configuration ---

# --- Webhook Dispatcher Logic ---

async def handle_dispatch_message(request: web.Request):
    try:
        if request.headers.get("X-API-KEY") != SERVICE_TOKEN:
            return web.Response(status=403, text="Forbidden")

        data = await request.json()
        bot_name = data.get("bot_name")
        telegram_id = data.get("telegram_id")
        message = data.get("message")
        message_to_edit = data.get("message_to_edit")
        message_to_delete = data.get("message_to_delete")
        inline_keyboard = data.get("inline_keyboard")
        
        logging.info(f"Received dispatch message for bot '{bot_name}' with data: {data}")

        if not all([bot_name, telegram_id, message]):
            return web.Response(status=400, text="Bad Request: missing fields")

        redis_client = request.app['redis']
        channel = f"bot-notifications:{bot_name}"
        
        payload_to_redis = {
            "telegram_id": telegram_id, 
            "message": message,
            "message_to_edit": message_to_edit,
            "message_to_delete": message_to_delete,
            "inline_keyboard": inline_keyboard
        }
        payload = json.dumps(payload_to_redis)

        await redis_client.publish(channel, payload)
        logging.info(f"Dispatched message to bot '{bot_name}' on channel '{channel}'")
        return web.Response(status=200, text="OK")

    except Exception as e:
        logging.exception("Error in handle_dispatch_message")
        return web.Response(status=500, text="Internal Server Error")

async def start_webhook_server(redis_client):
    app = web.Application()
    app['redis'] = redis_client
    app.router.add_post("/webhook/dispatch-message", handle_dispatch_message)
    
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, WEBHOOK_HOST, WEBHOOK_PORT)
    logging.info(f"Starting webhook dispatcher server on {WEBHOOK_HOST}:{WEBHOOK_PORT}")
    await site.start()

# --- Helper functions ---

def requests_session() -> requests.Session:
    session = requests.Session()
    session.headers.update({"X-API-KEY": SERVICE_TOKEN})
    retry = Retry(total=3, backoff_factor=0.5, status_forcelist=[500, 502, 503, 504])
    adapter = HTTPAdapter(max_retries=retry)
    session.mount('https://', adapter)
    session.mount('http://', adapter)
    return session

def get_bot_info(token: str) -> (dict | None, bool, bool):
    """Checks bot health. Returns (info, is_invalid, is_temporary_error)."""
    url = f"https://api.telegram.org/bot{token}/getMe"
    try:
        response = requests.get(url, timeout=15)
        if response.status_code == 200 and response.json().get("ok"):
            logging.info(f"Health check passed for token ...{token[-4:]}")
            return response.json()["result"], False, False
        
        if response.status_code in [401, 403, 404]: # 404 means token was revoked
            logging.warning(f"Token ...{token[-4:]} is invalid with status {response.status_code}. This is a permanent error.")
            return None, True, False

        logging.warning(f"Health check failed for token ...{token[-4:]} with status {response.status_code}. This is a temporary error.")
        return None, False, True

    except requests.exceptions.RequestException as e:
        logging.error(f"Health check network exception for ...{token[-4:]}: {e}. This is a temporary error.")
        return None, False, True

# --- Referral Bot Logic ---

def get_all_bots_from_api(session: requests.Session, bot_type: str) -> list:
    params = {
        "filters[0][field]": "type",
        "filters[0][op]": "eq",
        "filters[0][value]": bot_type,
    }
    url = f"{API_URL}/bots"
    try:
        response = session.get(url, params=params)
        response.raise_for_status()
        data = response.json()
        return data.get("items", [])
    except requests.exceptions.RequestException as e:
        logging.error(f"Failed to get all {bot_type} bots from API: {e}")
        return []

def set_bot_status_api(session: requests.Session, bot_id: int, is_active: bool):
    url = f"{API_URL}/bots/{bot_id}/status"
    try:
        response = session.put(url, json={"is_active": is_active})
        response.raise_for_status()
        logging.info(f"Set bot ID {bot_id} active status to {is_active} via API.")
    except requests.exceptions.RequestException as e:
        logging.error(f"Failed to set bot status for ID {bot_id} via API: {e}")

async def manage_referral_bots():
    logging.info("Starting referral bot management task...")
    running_procs = {}
    session = requests_session()
    while True:
        try:
            api_bots = get_all_bots_from_api(session, "referral") or []
            logging.info(f"Found {len(api_bots)} referral bots. Checking for healthy bots...")
            bots_by_owner = {}
            for bot in api_bots:
                if not bot.get("is_active"):
                    continue
                owner_id = bot.get("owner_id")
                if owner_id not in bots_by_owner:
                    bots_by_owner[owner_id] = []
                bots_by_owner[owner_id].append(bot)

            running_owners = set(running_procs.keys())
            for owner_id in running_owners:
                proc, running_bot_id = running_procs[owner_id]
                owner_bots = bots_by_owner.get(owner_id, [])
                running_bot_still_active = any(b['id'] == running_bot_id and b['is_active'] for b in owner_bots)
                if not owner_bots or not running_bot_still_active:
                    logging.info(f"Stopping bot process for owner {owner_id} (bot {running_bot_id}) as it's no longer active/valid.")
                    proc.kill()
                    del running_procs[owner_id]

            for owner_id, owner_bots in bots_by_owner.items():
                if owner_id in running_procs:
                    proc, running_bot_id = running_procs[owner_id]
                    if proc.poll() is not None:
                        logging.error(f"Referral bot ID {running_bot_id} for owner {owner_id} terminated unexpectedly.")
                        set_bot_status_api(session, running_bot_id, False)
                        del running_procs[owner_id]
                    else:
                        running_bot_token = next((b['bot_token'] for b in owner_bots if b['id'] == running_bot_id), None)
                        if not running_bot_token:
                            is_invalid = True # No token found, treat as invalid
                        else:
                            _, is_invalid, _ = get_bot_info(running_bot_token)

                        if is_invalid:
                            logging.warning(f"Health check failed for running bot ID {running_bot_id}. Killing process.")
                            proc.kill()
                            set_bot_status_api(session, running_bot_id, False)
                            del running_procs[owner_id]
                    if owner_id in running_procs:
                        continue

                primary_bot = next((b for b in owner_bots if b.get("is_primary")), None)
                bot_to_start = None
                if primary_bot:
                    info, is_invalid, _ = get_bot_info(primary_bot.get("bot_token"))
                    if info:
                        bot_to_start = primary_bot
                        logging.info(f"Primary bot ID {bot_to_start['id']} for owner {owner_id} is healthy. Will attempt to start.")
                    elif is_invalid:
                         logging.warning(f"Primary bot ID {primary_bot['id']} for owner {owner_id} is invalid.")
                
                if not bot_to_start:
                    if primary_bot:
                        logging.warning(f"Primary bot ID {primary_bot['id']} for owner {owner_id} failed health check. Looking for reserve.")
                    for reserve_bot in sorted(owner_bots, key=lambda b: b.get('id')):
                        if not reserve_bot.get("is_primary"):
                            info, is_invalid, _ = get_bot_info(reserve_bot.get("bot_token"))
                            if info:
                                bot_to_start = reserve_bot
                                logging.info(f"Found healthy reserve bot ID {bot_to_start['id']} for owner {owner_id}. Will attempt to start.")
                                break
                            elif is_invalid:
                                logging.warning(f"Reserve bot ID {reserve_bot['id']} is invalid, skipping.")
                
                if bot_to_start:
                    logging.info(f"Starting process for bot ID {bot_to_start['id']} (Owner: {owner_id}).")
                    env = os.environ.copy()
                    env["BOT_TOKEN"] = bot_to_start["bot_token"]
                    env["BOT_ID"] = str(bot_to_start["id"])
                    if "FALLBACK_BOT_USERNAME" in env:
                        del env["FALLBACK_BOT_USERNAME"]
                    proc = subprocess.Popen(BOT_COMMAND, env=env, cwd=str(Path(__file__).parent))
                    running_procs[owner_id] = (proc, bot_to_start['id'])

        except Exception as e:
            logging.exception("An error occurred in the referral bot management loop.")
        await asyncio.sleep(HEALTH_CHECK_INTERVAL)

# --- Main Bot Logic ---

def get_main_bots_from_api(session: requests.Session) -> list:
    params = {
        "filters[0][field]": "type",
        "filters[0][op]": "eq",
        "filters[0][value]": "main",
    }

    url = f"{API_URL}/bots?{urlencode(params)}"
    try:
        response = session.get(url)
        response.raise_for_status()
        data = response.json()
        return data.get("items", [])
    except requests.exceptions.RequestException as e:
        logging.error(f"Failed to get main bots from API: {e}")
        return []

def create_main_bot_in_api(session: requests.Session, token: str, username: str) -> bool:
    url = f"{API_URL}/bots/main"
    try:
        response = session.post(url, json={"token": token, "username": username})
        response.raise_for_status()
        logging.info(f"Successfully created main bot @{username} via API.")
        return True
    except requests.exceptions.RequestException as e:
        logging.error(f"Failed to create main bot @{username} via API: {e}")
        return False

async def request_new_main_bot_token() -> bool:
    if not API_ID or not API_HASH or API_ID == "YOUR_API_ID" or API_HASH == "YOUR_API_HASH":
        logging.error("API_ID and API_HASH are not set. Cannot request a new main bot.")
        return False

    logging.info("Attempting to create a new bot via BotFather.")
    try:
        async with TelegramClient(SESSION_NAME, API_ID, API_HASH) as client:
            async with client.conversation('BotFather', timeout=60) as conv:
                await conv.send_message('/newbot')
                resp = await conv.get_response()
                if 'Alright, a new bot.' not in resp.text:
                    logging.error(f"BotFather responded unexpectedly: {resp.text}")
                    await conv.cancel_all()
                    return False

                bot_name = f"My Monitored Bot {int(time.time())}"
                await conv.send_message(bot_name)
                resp = await conv.get_response()
                if 'Good. Now let\'s choose a username' not in resp.text:
                    logging.error(f"BotFather responded unexpectedly after name: {resp.text}")
                    return False

                while True:
                    bot_username = f"my_monitored_bot_{int(time.time())}_bot"
                    await conv.send_message(bot_username)
                    resp = await conv.get_response()
                    if "This username is already taken" in resp.text:
                        logging.warning(f"Username {bot_username} is taken. Trying another.")
                        await asyncio.sleep(4)
                        continue
                    elif 'Done! Congratulations' in resp.text:
                        token_match = re.search(r'(\d{9,10}:[a-zA-Z0-9_-]{35})', resp.text)
                        if token_match:
                            new_token = token_match.group(1).strip()
                            logging.info(f"Successfully created a new bot with token ...{new_token[-4:]}")
                            session = requests_session()
                            return create_main_bot_in_api(session, new_token, bot_username)
                        else:
                            logging.error("Could not parse the new bot token.")
                            return False
                    else:
                        logging.error(f"BotFather responded unexpectedly after username: {resp.text}")
                        return False
    except SessionPasswordNeededError:
        logging.error("Telethon session needs a 2FA password. Please run this script interactively first.")
        return False
    except Exception as e:
        logging.error(f"An error occurred with Telethon: {e}")
        return False

async def manage_main_bots():
    logging.info("Starting main bot management task...")
    bot_process = None
    session = requests_session()
    try:
        while True:
            all_bots = get_main_bots_from_api(session)
            healthy_bots = deque()
            logging.info(f"Found {len(all_bots)} main bots. Checking for healthy bots...")
            for bot in all_bots:
                info, is_invalid, _ = get_bot_info(bot['token'])
                if info:
                    healthy_bots.append({'token': bot['token'], 'info': info, 'id': bot['id']})
                elif is_invalid:
                    set_bot_status_api(session, bot['id'], False)

            if not healthy_bots:
                logging.warning("No healthy main bots found. Requesting a new one.")
                success = await request_new_main_bot_token()
                if not success:
                    logging.error("Failed to create a new main bot. Will retry in 5 minutes.")
                    await asyncio.sleep(300)
                continue

            active_bot = healthy_bots.popleft()
            active_token = active_bot['token']
            active_bot_info = active_bot['info']
            active_bot_id = active_bot['id']
            logging.info(f"Assigning ACTIVE main bot: @{active_bot_info['username']} (...{active_token[-4:]})")

            env = os.environ.copy()
            env["BOT_TOKEN"] = active_token
            env["BOT_TYPE"] = "main"
            env["BOT_ID"] = str(active_bot_id)

            if healthy_bots:
                fallback_bot = healthy_bots[0]
                fallback_bot_info = fallback_bot['info']
                logging.info(f"Assigning FALLBACK main bot: @{fallback_bot_info['username']}")
                env["FALLBACK_BOT_USERNAME"] = fallback_bot_info['username']
            else:
                logging.warning("Only one healthy main bot found. No fallback will be assigned.")
                if "FALLBACK_BOT_USERNAME" in env:
                    del env["FALLBACK_BOT_USERNAME"]

            bot_process = subprocess.Popen(BOT_COMMAND, env=env, cwd=str(Path(__file__).parent))
            await asyncio.sleep(STARTUP_WAIT_TIME)

            if bot_process.poll() is not None:
                logging.error(f"Main bot @{active_bot_info['username']} terminated on startup. This is likely a code issue, not a token problem. Will retry after interval.")
                continue

            logging.info(f"Successfully started and monitoring main bot @{active_bot_info['username']}.")

            while True:
                _, is_invalid, _ = get_bot_info(active_token)
                if is_invalid:
                    logging.warning(f"Main bot @{active_bot_info['username']} is not healthy. Killing process.")
                    bot_process.kill()
                    set_bot_status_api(session, active_bot_id, False)
                    break
                if bot_process.poll() is not None:
                    logging.error(f"Main bot @{active_bot_info['username']} has terminated unexpectedly.")
                    set_bot_status_api(session, active_bot_id, False)
                    break
                await asyncio.sleep(HEALTH_CHECK_INTERVAL)
    finally:
        if bot_process and bot_process.poll() is None:
            logging.info("Shutting down: Stopping main bot process...")
            bot_process.kill()

# --- Main Entry Point ---

async def main():
    setup_logging()
    redis_client = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)
    
    logging.info("Starting all monitor tasks...")
    await asyncio.gather(
        start_webhook_server(redis_client),
        manage_main_bots(),
        manage_referral_bots()
    )

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nMonitoring stopped by user.")