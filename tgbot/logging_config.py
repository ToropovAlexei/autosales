import logging
from logging.handlers import RotatingFileHandler
import sys

def setup_logging():
    """Настраивает основную конфигурацию логирования."""
    log_formatter = logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - %(message)s (%(filename)s:%(lineno)d)'
    )
    log_file = 'bot.log'

    # Настраиваем файловый обработчик с ротацией
    file_handler = RotatingFileHandler(
        log_file, maxBytes=5*1024*1024, backupCount=2  # 5 MB per file, 2 backups
    )
    file_handler.setFormatter(log_formatter)
    file_handler.setLevel(logging.INFO)

    # Настраиваем обработчик для вывода в консоль
    stdout_handler = logging.StreamHandler(sys.stdout)
    stdout_handler.setFormatter(log_formatter)
    stdout_handler.setLevel(logging.INFO)

    # Получаем корневой логгер и настраиваем его
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.INFO)
    root_logger.addHandler(file_handler)
    root_logger.addHandler(stdout_handler)

    # Применяем настройки и к логгеру aiogram
    logging.getLogger("aiogram").setLevel(logging.INFO)

    logging.info("Logging setup complete.")
