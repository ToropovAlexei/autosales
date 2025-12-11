package models

import "time"

type Role struct {
	ID          uint         `gorm:"primaryKey" json:"id"`
	Name        string       `gorm:"unique;not null" json:"name"`
	IsSuper     bool         `gorm:"default:false" json:"is_super"`
	CreatedAt   time.Time    `gorm:"autoCreateTime" json:"created_at"`
	Permissions []Permission `gorm:"many2many:role_permissions;" json:"permissions,omitempty"`
}

type Permission struct {
	ID    uint   `gorm:"primaryKey" json:"id"`
	Name  string `gorm:"unique;not null" json:"name"`
	Group string `gorm:"not null" json:"group"`
}

type RolePermission struct {
	RoleID       uint `gorm:"primaryKey" json:"role_id"`
	PermissionID uint `gorm:"primaryKey" json:"permission_id"`
}

type UserRole struct {
	UserID uint `gorm:"primaryKey" json:"user_id"`
	RoleID uint `gorm:"primaryKey" json:"role_id"`
}

type UserPermission struct {
	UserID       uint   `gorm:"primaryKey" json:"user_id"`
	PermissionID uint   `gorm:"primaryKey" json:"permission_id"`
	Effect       string `gorm:"not null;default:'allow'" json:"effect"` // allow, deny
}
