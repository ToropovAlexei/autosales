from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    bot_token: str
    api_url: str
    service_token: str
    redis_host: str
    redis_port: int
    support_url: str
    api_id: str
    api_hash: str

    class Config:
        env_file = ".env"

settings = Settings()
