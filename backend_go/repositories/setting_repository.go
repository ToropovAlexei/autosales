package repositories

import (
	"frbktg/backend_go/models"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

type SettingRepository struct {
	db *gorm.DB
}

func NewSettingRepository(db *gorm.DB) *SettingRepository {
	return &SettingRepository{db: db}
}

func (r *SettingRepository) GetSettings() ([]models.Setting, error) {
	var settings []models.Setting
	err := r.db.Find(&settings).Error
	return settings, err
}

func (r *SettingRepository) UpsertSettings(settings []models.Setting) error {
	if len(settings) == 0 {
		return nil
	}
	return r.db.Clauses(clause.OnConflict{
		Columns:   []clause.Column{{Name: "key"}},
		DoUpdates: clause.AssignmentColumns([]string{"value"}),
	}).Create(&settings).Error
}
