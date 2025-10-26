package models

type User struct {
	ID                     uint      `gorm:"primaryKey" json:"id"`
	Email                  string    `gorm:"uniqueIndex" json:"email"`
	HashedPassword         string    `json:"-"` // Do not expose hashed password
	IsActive               bool      `gorm:"default:true" json:"is_active"`
	TwoFASecret            *string   `json:"-"`
	TwoFAEnabled           bool      `gorm:"default:true" json:"two_fa_enabled"`
	ReferralProgramEnabled bool      `gorm:"default:false" json:"referral_program_enabled"`
	ReferralPercentage     float64   `gorm:"default:0.0" json:"referral_percentage"`
	Roles                  []*Role   `gorm:"many2many:user_roles;" json:"roles,omitempty"`
	Permissions            []*UserPermission `json:"permissions,omitempty"`
}

type UserResponse struct {
	ID                     uint     `json:"id"`
	Email                  string   `json:"email"`
	IsActive               bool     `json:"is_active"`
	ReferralProgramEnabled bool     `json:"referral_program_enabled"`
	ReferralPercentage     float64  `json:"referral_percentage"`
	Roles                  []*Role  `json:"roles,omitempty"`
}

type CreateUserResponse struct {
	User      UserResponse `json:"user"`
	TwoFASecret string       `json:"two_fa_secret"`
	QRCode    string       `json:"qr_code"`
}
