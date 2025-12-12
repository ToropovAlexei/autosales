package services

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"encoding/base64"
	"errors"
	"fmt"
	"io"
	"time"

	"github.com/pquerna/otp"
	"github.com/pquerna/otp/totp"
	"github.com/skip2/go-qrcode"
)

type TwoFAService interface {
	GenerateSecret(login string) (string, error)
	EncryptSecret(secret string) (string, error)
	DecryptSecret(encryptedSecret string) (string, error)
	GenerateQRCode(login, secret string) ([]byte, error)
	ValidateCode(secret, code string) bool
}

type twoFAService struct {
	secretKey []byte
}

func NewTwoFAService(secretKey string) (TwoFAService, error) {
	key := []byte(secretKey)
	if len(key) != 32 {
		return nil, errors.New("secret key must be 32 bytes, got " + fmt.Sprint(len(key)))
	}
	return &twoFAService{secretKey: key}, nil
}

func (s *twoFAService) GenerateSecret(login string) (string, error) {
	key, err := totp.Generate(totp.GenerateOpts{
		Issuer:      "YourAppName",
		AccountName: login,
	})
	if err != nil {
		return "", err
	}
	return key.Secret(), nil
}

func (s *twoFAService) EncryptSecret(secret string) (string, error) {
	block, err := aes.NewCipher(s.secretKey)
	if err != nil {
		return "", err
	}

	plaintext := []byte(secret)
	ciphertext := make([]byte, aes.BlockSize+len(plaintext))
	iv := ciphertext[:aes.BlockSize]
	if _, err := io.ReadFull(rand.Reader, iv); err != nil {
		return "", err
	}

	stream := cipher.NewCFBEncrypter(block, iv)
	stream.XORKeyStream(ciphertext[aes.BlockSize:], plaintext)

	return base64.URLEncoding.EncodeToString(ciphertext), nil
}

func (s *twoFAService) DecryptSecret(encryptedSecret string) (string, error) {
	ciphertext, err := base64.URLEncoding.DecodeString(encryptedSecret)
	if err != nil {
		return "", err
	}

	block, err := aes.NewCipher(s.secretKey)
	if err != nil {
		return "", err
	}

	if len(ciphertext) < aes.BlockSize {
		return "", errors.New("ciphertext too short")
	}
	iv := ciphertext[:aes.BlockSize]
	ciphertext = ciphertext[aes.BlockSize:]

	stream := cipher.NewCFBDecrypter(block, iv)
	stream.XORKeyStream(ciphertext, ciphertext)

	return fmt.Sprintf("%s", ciphertext), nil
}

func (s *twoFAService) GenerateQRCode(login, secret string) ([]byte, error) {
	url := fmt.Sprintf("otpauth://totp/YourAppName:%s?secret=%s&issuer=YourAppName", login, secret)
	return qrcode.Encode(url, qrcode.Medium, 256)
}

func (s *twoFAService) ValidateCode(secret, code string) bool {
	valid, err := totp.ValidateCustom(
		code,
		secret,
		time.Now().UTC(),
		totp.ValidateOpts{
			Period:    30,
			Skew:      1,
			Digits:    otp.DigitsSix,
			Algorithm: otp.AlgorithmSHA1,
		},
	)
	if err != nil {
		return false
	}
	return valid
}
