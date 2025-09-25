from pydantic_settings import BaseSettings
from pydantic import model_validator
from typing import Optional

class Settings(BaseSettings):
    bot_token: str
    bot_type: str = "main"
    fallback_bot_username: Optional[str] = None
    api_url: str
    service_token: str
    redis_host: str
    redis_port: int
    support_url: str
    api_id: str
    api_hash: str

    @model_validator(mode='after')
    def set_bot_type(self) -> 'Settings':
        if self.fallback_bot_username:
            self.bot_type = "main"
        else:
            self.bot_type = "referral"
        return self

    class Config:
        env_file = ".env"

settings = Settings()
