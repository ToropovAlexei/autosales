package models

type Login2FAPayload struct {
	Email    string `json:"email" binding:"required,email"`
	Password string `json:"password"`
	Code     string `json:"code"`
}
