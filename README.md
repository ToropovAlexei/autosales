# E-commerce Platform with Go, Next.js, and a Telegram Bot

This is a comprehensive e-commerce platform application. It includes a backend built with Go (Gin), an administration frontend using Next.js, and a customer-facing Telegram bot using Python's `aiogram`.

## Features

- **Admin Panel:** A user-friendly web interface for managing categories, products, stock, orders, and system settings.
- **Telegram Bot for Customers:** Allows users to browse the product catalog, manage their balance, and make purchases directly within Telegram.
- **External Provider Integration:** The system can synchronize products from external providers (like proxy services) and manage subscriptions.
- **Referral Program:** Users can create their own referral "bot shops" to earn a commission on sales.
- **High-Availability Bot System:** The main Telegram bot is managed by a monitor that tracks its status. In case of a failure, it can switch to a fallback bot to ensure uninterrupted service.

## Prerequisites

Before you begin, ensure you have the following installed:

- Go 1.21+
- Node.js 18.x+
- pnpm (or npm/yarn)
- Python 3.8+
- Docker and Docker Compose

## Database Setup

For local development, the project uses PostgreSQL and Redis, which can be run via Docker Compose.

1.  **Start the containers:**
    From the root directory of the project, run:
    ```bash
    docker-compose up -d
    ```
    This command will start PostgreSQL and Redis services in the background.

## Installation and Running

The project is divided into three main components: `backend_go`, `frontend`, and `tgbot`. Follow the instructions below to set up each part.

### Backend (`backend_go`)

The backend is a Go application using the Gin framework, serving as a REST API for the frontend and the Telegram bot.

1.  **Navigate to the backend directory:**

    ```bash
    cd backend_go
    ```

2.  **Create `.env` file:**
    Copy the contents of `.env.example` into a new `.env` file and fill in the necessary configuration details (database connection, secrets, etc.).

3.  **Install dependencies:**

    ```bash
    go mod tidy
    ```

4.  **Run the backend server:**
    ```bash
    go run main.go
    ```
    The backend will start on `http://127.0.0.1:8000` (or the port specified in your `.env`). The server will automatically handle database migrations on startup.

### Frontend (`frontend`)

The frontend is a Next.js application that provides the administration panel.

1.  **Navigate to the frontend directory:**

    ```bash
    cd frontend
    ```

2.  **Install dependencies:**

    ```bash
    pnpm install
    ```

3.  **Run the development server:**
    ```bash
    pnpm run dev
    ```
    The frontend will be available at `http://localhost:3000`.

### Telegram Bot (`tgbot`)

The Telegram bot is built with `aiogram` and serves as the primary client interface.

#### Key Concepts

To configure and understand the bot's operation, it's important to be familiar with these concepts:

- **Main and Fallback Bots:** The system is designed for high availability. It uses at least two bots: a **main** (active) bot and a **fallback** (standby) bot. The `monitor.py` script continuously checks the health of the main bot. If it becomes unresponsive, the monitor promotes the fallback bot to main, ensuring service continuity.

- **Automatic Bot Creation (`API_ID` & `API_HASH`):** For the system to be self-healing, it needs permission to create new bots on your behalf. This is done using your personal Telegram developer keys, `API_ID` and `API_HASH`.
  - **What are they?** These are not bot tokens, but credentials for your personal Telegram account. They allow the `monitor.py` script to connect to Telegram as you and perform actions like creating a new bot via `BotFather` if the number of healthy bots drops below a threshold.
  - **How to get them:** You can obtain them from the official [my.telegram.org](https://my.telegram.org) website by logging in and navigating to the "API development tools" section.

#### Setup and Launch

1.  **Create Bots:**

    - Open Telegram and find the official `BotFather` bot.
    - Create at least two bots using the `/newbot` command.
    - Save the tokens for both bots. A token looks like `1234567890:ABC-DEF1234ghIkl-zyx57W2v1u1234567890`.

2.  **Navigate to the `tgbot` directory:**

    ```bash
    cd tgbot
    ```

3.  **Fill `tokens.txt`:**

    - Create a file named `tokens.txt`.
    - Paste the tokens of the two bots you created, each on a new line. The monitor will use this list to manage the bots.

4.  **Fill in the `.env` file:**

    - Copy `.env.example` to a new `.env` file.
    - Enter your `API_ID` and `API_HASH` obtained from [my.telegram.org](https://my.telegram.org).
    - Fill in the other variables, such as `SERVICE_TOKEN` (for backend communication) and Redis settings.
    - **Note:** The `BOT_TOKEN` in this file will be overwritten by the monitor but can be left for testing purposes.

5.  **Create and activate a virtual environment:**

    ```bash
    python -m venv .venv
    source .venv/bin/activate
    ```

6.  **Install dependencies:**

    ```bash
    pip install -r requirements.txt
    ```

7.  **Run the bot monitor:**
    ```bash
    python monitor.py
    ```
    The monitor will launch the main bot from your list, designate a fallback, and keep them operational. If a bot fails, the monitor will automatically promote the fallback and, if necessary, create a new one using your `API_ID` and `API_HASH`.
