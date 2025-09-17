package models



type UserRole string

const (
	Admin  UserRole = "admin"
	Seller UserRole = "seller"
)

type User struct {
	ID                      uint      `gorm:"primaryKey"`
	Email                   string    `gorm:"uniqueIndex"`
	HashedPassword          string
	IsActive                bool      `gorm:"default:true"`
	Role                    UserRole  `gorm:"default:seller;not null"`
	ReferralProgramEnabled  bool      `gorm:"default:false"`
	ReferralPercentage      float64   `gorm:"default:0.0"`
}

type UserResponse struct {
	ID                     uint           `json:"id"`
	Email                  string         `json:"email"`
	IsActive               bool           `json:"is_active"`
	Role                   UserRole `json:"role"`
	ReferralProgramEnabled bool           `json:"referral_program_enabled"`
	ReferralPercentage     float64        `json:"referral_percentage"`
}
