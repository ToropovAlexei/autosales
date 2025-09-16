package responses

import "frbktg/backend_go/models"

type UserResponse struct {
	ID                     uint           `json:"id"`
	Email                  string         `json:"email"`
	IsActive               bool           `json:"is_active"`
	Role                   models.UserRole `json:"role"`
	ReferralProgramEnabled bool           `json:"referral_program_enabled"`
	ReferralPercentage     float64        `json:"referral_percentage"`
}
