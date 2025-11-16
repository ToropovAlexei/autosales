package models

import (
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Image struct {
	ID               uuid.UUID `gorm:"type:uuid;primary_key;"`
	OriginalFilename string
	Hash             string `gorm:"uniqueIndex"` // SHA256 hash of the file content
	MimeType         string
	FileSize         int64
	Folder           string `gorm:"index"`
	CreatedAt        time.Time
	DeletedAt        gorm.DeletedAt `gorm:"index" json:"-"`
}

// BeforeCreate will set a UUID rather than an integer ID.
func (image *Image) BeforeCreate(tx *gorm.DB) (err error) {
	if image.ID == uuid.Nil {
		image.ID = uuid.New()
	}
	return
}
