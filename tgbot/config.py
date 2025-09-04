from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    bot_token: str = "868666671:AAFEequl_hLLxd_J0cz1TsolQExIvpzsaEQ"
    api_url: str = "http://127.0.0.1:8000/api"
    service_token: str = "a_very_secret_service_key"
    redis_host: str = "localhost"
    redis_port: int = 6379
    support_url: str = "http://localhost:8000/support"

    class Config:
        env_file = ".env"

settings = Settings()
