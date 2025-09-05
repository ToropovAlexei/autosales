import asyncio
import logging
import os
import subprocess
import time
from pathlib import Path

import requests

# --- Configuration ---
TOKENS_FILE = Path("tokens.txt")
UNAVAILABLE_TOKENS_FILE = Path("unavailable_tokens.txt")
BOT_COMMAND = ["python", "main.py"]
HEALTH_CHECK_INTERVAL = 60
STARTUP_WAIT_TIME = 5 # seconds to wait for the bot to start before the first check
# --- End Configuration ---

logging.basicConfig(level=logging.INFO)

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
        # Add to unavailable file
        with open(UNAVAILABLE_TOKENS_FILE, "a") as f:
            f.write(f"{token}\n")

        # Remove from available file
        tokens = load_tokens()
        tokens = [t for t in tokens if t != token]
        with open(TOKENS_FILE, "w") as f:
            for t in tokens:
                f.write(f"{t}\n")
    except IOError as e:
        logging.error(f"Error updating token files: {e}")

def is_bot_healthy(token: str) -> bool:
    """Checks if the bot is healthy by calling the getMe method."""
    url = f"https://api.telegram.org/bot{token}/getMe"
    try:
        response = requests.get(url, timeout=10)
        if response.status_code == 200:
            logging.info("Health check passed.")
            return True
        logging.warning(f"Health check failed with status code {response.status_code}: {response.text}")
        return False
    except requests.exceptions.RequestException as e:
        logging.error(f"Health check failed with an exception: {e}")
        return False

async def main():
    """
    The main function that starts and monitors the bot.
    """
    while True:
        available_tokens = load_tokens()
        if not available_tokens:
            logging.info(f"No available tokens in {TOKENS_FILE}. Checking again in 60 seconds.")
            await asyncio.sleep(60)
            continue

        for token in available_tokens:
            logging.info(f"Starting bot with token ...{token[-4:]}")

            env = os.environ.copy()
            env["BOT_TOKEN"] = token

            bot_process = subprocess.Popen(BOT_COMMAND, env=env, cwd="/home/user/repos/frbktg/tgbot")

            await asyncio.sleep(STARTUP_WAIT_TIME)

            # Check if the process terminated during startup
            if bot_process.poll() is not None:
                logging.error("Bot process terminated on startup. Marking token as unavailable.")
                mark_token_as_unavailable(token)
                continue  # Move to the next token

            # If startup seems ok, enter the main monitoring loop for this token
            while True:
                if not is_bot_healthy(token):
                    logging.warning("Bot is not healthy. Killing process and moving to the next token.")
                    bot_process.kill()
                    mark_token_as_unavailable(token)
                    break  # Exit inner loop to switch token

                # Check if the process died since the last health check
                if bot_process.poll() is not None:
                    logging.error("Bot process has terminated unexpectedly. Marking token as unavailable.")
                    mark_token_as_unavailable(token)
                    break # Exit inner loop to switch token

                await asyncio.sleep(HEALTH_CHECK_INTERVAL)

if __name__ == "__main__":
    if not TOKENS_FILE.exists() or not TOKENS_FILE.read_text().strip():
        print(f"The file {TOKENS_FILE} is missing or empty.")
        print(f"Please create it and add your bot tokens, one per line.")
    else:
        asyncio.run(main())
