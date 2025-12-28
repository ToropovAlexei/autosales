use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, Payload, consts::U12},
};
use base64::Engine;
use base64::engine::general_purpose;

use crate::errors::totp_encryptor::{TotpEncryptorError, TotpEncryptorResult};

pub struct TotpEncryptor {
    cipher: Aes256Gcm,
}

impl TotpEncryptor {
    pub fn new(key_b64: &str) -> TotpEncryptorResult<Self> {
        let key_bytes = general_purpose::STANDARD
            .decode(key_b64)
            .map_err(|_e| TotpEncryptorError::MissingSecret)?;
        if key_bytes.len() != 32 {
            return Err(TotpEncryptorError::MissingSecret);
        }
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        Ok(Self { cipher })
    }

    fn nonce_for_user(user_id: i64) -> Nonce<U12> {
        let mut bytes = [0u8; 12];
        bytes[0..8].copy_from_slice(&user_id.to_be_bytes());
        *Nonce::from_slice(&bytes)
    }

    pub fn encrypt(&self, user_id: i64, secret: &str) -> TotpEncryptorResult<String> {
        let nonce_bytes = Self::nonce_for_user(user_id);
        let ciphertext: Vec<u8> = self
            .cipher
            .encrypt(&nonce_bytes, Payload::from(secret.as_bytes()))
            .map_err(|_e| TotpEncryptorError::DecodeError("Failed to encrypt".to_string()))?;
        Ok(general_purpose::STANDARD.encode(&ciphertext))
    }

    pub fn decrypt(&self, user_id: i64, encrypted_b64: &str) -> TotpEncryptorResult<String> {
        let nonce = Self::nonce_for_user(user_id);
        let ciphertext = general_purpose::STANDARD
            .decode(encrypted_b64)
            .map_err(|_e| TotpEncryptorError::DecodeError("Failed to decode".to_string()))?;
        let plaintext = self
            .cipher
            .decrypt(&nonce, Payload::from(&ciphertext[..]))
            .map_err(|_e| TotpEncryptorError::DecodeError("Failed to decrypt".to_string()))?;
        String::from_utf8(plaintext)
            .map_err(|_e| TotpEncryptorError::DecodeError("Failed to decode".to_string()))
    }
}
