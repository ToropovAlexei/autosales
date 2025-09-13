import asyncio
import logging
import os
import re
import subprocess
import time
from pathlib import Path
from dotenv import load_dotenv
from collections import deque

import requests

load_dotenv()
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry
from telethon.sync import TelegramClient
from telethon.errors import SessionPasswordNeededError

# --- Configuration ---
# Telegram API credentials for creating new bots. Get them from my.telegram.org
API_ID = os.getenv("API_ID")
API_HASH = os.getenv("API_HASH")
SESSION_NAME = "bot_creator"

TOKENS_FILE = Path("tokens.txt")
UNAVAILABLE_TOKENS_FILE = Path("unavailable_tokens.txt")
BOT_COMMAND = ["python", "main.py"]
HEALTH_CHECK_INTERVAL = 60
STARTUP_WAIT_TIME = 10 # seconds to wait for the bot to start before the first check
# --- End Configuration ---

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

def load_tokens() -> list[str]:
    """Loads tokens from the tokens file."""
    if not TOKENS_FILE.exists():
        return []
    try:
        with open(TOKENS_FILE, "r") as f:
            tokens = [line.strip() for line in f if line.strip()]
        return tokens
    except IOError as e:
        logging.error(f"Error reading {TOKENS_FILE}: {e}")
        return []

def mark_token_as_unavailable(token: str):
    """Moves a token to the unavailable file."""
    logging.warning(f"Marking token ...{token[-4:]} as unavailable.")
    try:
        with open(UNAVAILABLE_TOKENS_FILE, "a") as f:
            f.write(f"{token}\n")
        
        current_tokens = load_tokens()
        if token in current_tokens:
            current_tokens.remove(token)
            with open(TOKENS_FILE, "w") as f:
                for t in current_tokens:
                    f.write(f"{t}\n")

    except IOError as e:
        logging.error(f"Error updating token files: {e}")

def get_bot_info(token: str) -> dict | None:
    """Checks bot health by calling getMe and returns bot info if healthy."""
    url = f"https://api.telegram.org/bot{token}/getMe"
    
    session = requests.Session()
    retry = Retry(
        total=3,
        backoff_factor=0.5,
        status_forcelist=[500, 502, 503, 504],
    )
    adapter = HTTPAdapter(max_retries=retry)
    session.mount('https://', adapter)

    try:
        response = session.get(url, timeout=15)
        if response.status_code == 200:
            data = response.json()
            if data.get("ok"):
                logging.info(f"Health check passed for token ...{token[-4:]}")
                return data["result"]
        
        if response.status_code == 401:
             logging.warning(f"Health check failed for token ...{token[-4:]}: Token is unauthorized.")
        else:
            logging.warning(f"Health check failed for token ...{token[-4:]} with status code {response.status_code}.")
        return None
    except requests.exceptions.RequestException as e:
        logging.error(f"Health check for token ...{token[-4:]} failed with a network exception: {e}")
        return None

async def request_new_bot_token() -> bool:
    """Interactively contacts BotFather to create a new bot and get a token."""
    if not API_ID or not API_HASH or API_ID == "YOUR_API_ID" or API_HASH == "YOUR_API_HASH":
        logging.error("API_ID and API_HASH are not set. Cannot request a new bot.")
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

async def main():
    """The main function that starts and monitors the bot."""
    bot_process = None
    try:
        while True:
            all_tokens = load_tokens()
            healthy_bots = deque()

            logging.info(f"Found {len(all_tokens)} tokens. Checking for healthy bots...")
            for token in all_tokens:
                info = get_bot_info(token)
                if info:
                    healthy_bots.append({'token': token, 'info': info})
                else:
                    mark_token_as_unavailable(token)
            
            while len(healthy_bots) < 2:
                logging.warning(f"Found only {len(healthy_bots)} healthy bots. Need at least 2 (active + fallback). Requesting a new one.")
                success = await request_new_bot_token()
                if not success:
                    logging.error("Failed to create a new bot. Will retry in 5 minutes.")
                    await asyncio.sleep(300)
                    continue
                
                # Reload tokens and re-evaluate health
                all_tokens = load_tokens()
                healthy_bots = deque()
                for token in all_tokens:
                    info = get_bot_info(token)
                    if info:
                        healthy_bots.append({'token': token, 'info': info})
                    else:
                        mark_token_as_unavailable(token)

            active_bot = healthy_bots.popleft()
            fallback_bot = healthy_bots[0]

            active_token = active_bot['token']
            active_bot_info = active_bot['info']
            fallback_bot_info = fallback_bot['info']

            logging.info(f"Assigning ACTIVE bot: @{active_bot_info['username']} (...{active_token[-4:]})")
            logging.info(f"Assigning FALLBACK bot: @{fallback_bot_info['username']}")

            env = os.environ.copy()
            env["BOT_TOKEN"] = active_token
            env["FALLBACK_BOT_USERNAME"] = fallback_bot_info['username']
            
            bot_process = subprocess.Popen(BOT_COMMAND, env=env, cwd="/home/user/repos/frbktg/tgbot")
            await asyncio.sleep(STARTUP_WAIT_TIME)

            if bot_process.poll() is not None:
                logging.error(f"Active bot @{active_bot_info['username']} terminated on startup. Marking as unavailable.")
                mark_token_as_unavailable(active_token)
                continue

            logging.info(f"Successfully started and monitoring bot @{active_bot_info['username']}.")

            while True:
                if get_bot_info(active_token) is None:
                    logging.warning(f"Active bot @{active_bot_info['username']} is not healthy. Killing process.")
                    bot_process.kill()
                    mark_token_as_unavailable(active_token)
                    break
                
                if bot_process.poll() is not None:
                    logging.error(f"Active bot @{active_bot_info['username']} has terminated unexpectedly.")
                    mark_token_as_unavailable(active_token)
                    break
                
                await asyncio.sleep(HEALTH_CHECK_INTERVAL)
    finally:
        if bot_process and bot_process.poll() is None:
            logging.info("Shutting down: Stopping the bot process...")
            bot_process.kill()

if __name__ == "__main__":
    try:
        if not API_ID or not API_HASH or API_ID == "YOUR_API_ID" or API_HASH == "YOUR_API_HASH":
            print("Please open monitor.py and fill in your API_ID and API_HASH from my.telegram.org")
        else:
            asyncio.run(main())
    except KeyboardInterrupt:
        print("\nMonitoring stopped by user.")