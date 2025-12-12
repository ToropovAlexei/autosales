package models

type Login2FAPayload struct {
	Login    string `json:"login" binding:"required"`
	Password string `json:"password"`
	Code     string `json:"code"`
}
