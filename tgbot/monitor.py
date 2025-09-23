import asyncio
import logging
import os
import re
import subprocess
import time
from pathlib import Path
from collections import deque

import requests
from dotenv import load_dotenv
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry
from telethon.sync import TelegramClient
from telethon.errors import SessionPasswordNeededError

from logging_config import setup_logging

# --- Конфигурация ---
load_dotenv()

# Для создания основных ботов через BotFather
API_ID = os.getenv("API_ID")
API_HASH = os.getenv("API_HASH")
SESSION_NAME = "bot_creator"

# Для работы с API бэкенда
API_URL = os.getenv("API_URL")
SERVICE_TOKEN = os.getenv("SERVICE_TOKEN")

# Общие настройки
TOKENS_FILE = Path("tokens.txt")
UNAVAILABLE_TOKENS_FILE = Path("unavailable_tokens.txt")
BOT_COMMAND = ["python", "main.py"]
HEALTH_CHECK_INTERVAL = 60
STARTUP_WAIT_TIME = 10
# --- Конец конфигурации ---


# --- Вспомогательные функции ---

def requests_session() -> requests.Session:
    session = requests.Session()
    session.headers.update({"X-API-KEY": SERVICE_TOKEN})
    retry = Retry(total=3, backoff_factor=0.5, status_forcelist=[500, 502, 503, 504])
    adapter = HTTPAdapter(max_retries=retry)
    session.mount('https://', adapter)
    session.mount('http://', adapter)
    return session

def get_bot_info(token: str) -> dict | None:
    url = f"https://api.telegram.org/bot{token}/getMe"
    try:
        response = requests.get(url, timeout=15)
        if response.status_code == 200 and response.json().get("ok"):
            logging.info(f"Health check passed for token ...{token[-4:]}")
            return response.json()["result"]
        logging.warning(f"Health check failed for token ...{token[-4:]} with status {response.status_code}")
        return None
    except requests.exceptions.RequestException as e:
        logging.error(f"Health check network exception for ...{token[-4:]}: {e}")
        return None

# --- Логика для реферальных ботов ---

def get_all_referral_bots_from_api(session: requests.Session) -> list:
    """Получает список ВСЕХ реферальных ботов с бэкенда."""
    url = f"{API_URL}/referrals"
    try:
        response = session.get(url)
        response.raise_for_status()
        data = response.json()
        return data.get("data", []) if data.get("success") else []
    except requests.exceptions.RequestException as e:
        logging.error(f"Failed to get all referral bots from API: {e}")
        return []

def set_bot_status_api(session: requests.Session, bot_id: int, is_active: bool):
    """Обновляет статус бота через API."""
    url = f"{API_URL}/referrals/{bot_id}/status"
    try:
        response = session.put(url, json={"is_active": is_active})
        response.raise_for_status()
        logging.info(f"Set bot ID {bot_id} active status to {is_active} via API.")
    except requests.exceptions.RequestException as e:
        logging.error(f"Failed to set bot status for ID {bot_id} via API: {e}")

async def manage_referral_bots():
    """Отслеживает, запускает и останавливает все активные основные реферальные боты."""
    logging.info("Starting referral bot management task...")
    running_procs = {}
    session = requests_session()

    while True:
        try:
            api_bots = get_all_referral_bots_from_api(session) or []
            bots_to_run = {
                b["id"]: b
                for b in api_bots
                if b.get("is_active") and b.get("is_primary")
            }
            running_bot_ids = set(running_procs.keys())

            for bot_id in running_bot_ids:
                if bot_id not in bots_to_run:
                    logging.info(f"Referral bot ID {bot_id} is no longer primary/active. Stopping process.")
                    running_procs[bot_id].kill()
                    del running_procs[bot_id]

            for bot_id, bot_data in bots_to_run.items():
                token = bot_data.get("bot_token")
                if not token:
                    continue

                if bot_id in running_procs:
                    if running_procs[bot_id].poll() is not None:
                        logging.error(f"Referral bot ID {bot_id} terminated unexpectedly. Marking as inactive.")
                        set_bot_status_api(session, bot_id, False)
                        del running_procs[bot_id]
                    continue

                if not get_bot_info(token):
                    logging.warning(f"Referral bot ID {bot_id} is primary/active but failed health check. Setting to inactive.")
                    set_bot_status_api(session, bot_id, False)
                    continue
                
                logging.info(f"Found new primary/active referral bot ID {bot_id}. Starting process.")
                env = os.environ.copy()
                env["BOT_TOKEN"] = token
                proc = subprocess.Popen(BOT_COMMAND, env=env, cwd=str(Path(__file__).parent))
                running_procs[bot_id] = proc

        except Exception as e:
            logging.exception("An error occurred in the referral bot management loop.")

        await asyncio.sleep(HEALTH_CHECK_INTERVAL)


# --- Логика для основных ботов (из tokens.txt) ---

def load_main_tokens() -> list[str]:
    if not TOKENS_FILE.exists(): return []
    try:
        with open(TOKENS_FILE, "r") as f: return [line.strip() for line in f if line.strip()]
    except IOError as e:
        logging.error(f"Error reading {TOKENS_FILE}: {e}")
        return []

def mark_main_token_as_unavailable(token: str):
    logging.warning(f"Marking main bot token ...{token[-4:]} as unavailable.")
    try:
        with open(UNAVAILABLE_TOKENS_FILE, "a") as f: f.write(f"{token}\n")
        current_tokens = load_main_tokens()
        if token in current_tokens:
            current_tokens.remove(token)
            with open(TOKENS_FILE, "w") as f:
                for t in current_tokens: f.write(f"{t}\n")
    except IOError as e:
        logging.error(f"Error updating main token files: {e}")

async def request_new_main_bot_token() -> bool:
    """Интерактивно связывается с BotFather для создания нового бота и получения токена."""
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
                            new_token = token_match.group(1)
                            logging.info(f"Successfully created a new bot with token ...{new_token[-4:]}")
                            with open(TOKENS_FILE, "a") as f:
                                f.write(f"{new_token}\n")
                            return True
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
    """Отслеживает и обеспечивает работу основных ботов из tokens.txt."""
    logging.info("Starting main bot management task...")
    bot_process = None
    try:
        while True:
            all_tokens = load_main_tokens()
            healthy_bots = deque()
            logging.info(f"Found {len(all_tokens)} main tokens. Checking for healthy bots...")
            for token in all_tokens:
                info = get_bot_info(token)
                if info: healthy_bots.append({'token': token, 'info': info})
                else: mark_main_token_as_unavailable(token)
            
            if len(healthy_bots) < 2:
                logging.warning(f"Found only {len(healthy_bots)} healthy main bots. Requesting a new one.")
                success = await request_new_main_bot_token()
                if not success: 
                    logging.error("Failed to create a new main bot. Will retry in 5 minutes.")
                    await asyncio.sleep(300)
                    continue
                continue # Перезапускаем цикл для переоценки

            active_bot = healthy_bots.popleft()
            fallback_bot = healthy_bots[0]
            active_token = active_bot['token']
            active_bot_info = active_bot['info']
            fallback_bot_info = fallback_bot['info']

            logging.info(f"Assigning ACTIVE main bot: @{active_bot_info['username']} (...{active_token[-4:]})")
            logging.info(f"Assigning FALLBACK main bot: @{fallback_bot_info['username']}")

            env = os.environ.copy()
            env["BOT_TOKEN"] = active_token
            env["FALLBACK_BOT_USERNAME"] = fallback_bot_info['username']
            
            bot_process = subprocess.Popen(BOT_COMMAND, env=env, cwd=str(Path(__file__).parent))
            await asyncio.sleep(STARTUP_WAIT_TIME)

            if bot_process.poll() is not None:
                logging.error(f"Main bot @{active_bot_info['username']} terminated on startup. Marking as unavailable.")
                mark_main_token_as_unavailable(active_token)
                continue

            logging.info(f"Successfully started and monitoring main bot @{active_bot_info['username']}.")

            while True:
                if get_bot_info(active_token) is None:
                    logging.warning(f"Main bot @{active_bot_info['username']} is not healthy. Killing process.")
                    bot_process.kill()
                    mark_main_token_as_unavailable(active_token)
                    break
                if bot_process.poll() is not None:
                    logging.error(f"Main bot @{active_bot_info['username']} has terminated unexpectedly.")
                    mark_main_token_as_unavailable(active_token)
                    break
                await asyncio.sleep(HEALTH_CHECK_INTERVAL)
    finally:
        if bot_process and bot_process.poll() is None:
            logging.info("Shutting down: Stopping main bot process...")
            bot_process.kill()

# --- Основная точка входа ---

async def main():
    setup_logging()
    # Запускаем обе задачи параллельно
    await asyncio.gather(
        manage_main_bots(),
        manage_referral_bots()
    )

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nMonitoring stopped by user.")