package models

import (
	"time"

	"gorm.io/datatypes"
)

// AuditLog represents a log entry for a user action.
type AuditLog struct {
	ID          uint           `gorm:"primaryKey" json:"id"`
	UserID      uint           `json:"user_id"`
	UserLogin   string         `json:"user_login"`
	Action      string         `gorm:"index" json:"action"`
	TargetType  string         `json:"target_type"`
	TargetID    uint           `json:"target_id"`
	Changes     datatypes.JSON `gorm:"type:jsonb" json:"changes"`
	Status      string         `json:"status"`
	IPAddress   string         `json:"ip_address"`
	UserAgent   string         `json:"user_agent"`
	CreatedAt   time.Time      `gorm:"autoCreateTime" json:"created_at"`
}
