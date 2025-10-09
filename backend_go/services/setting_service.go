package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type SettingService struct {
	repo             *repositories.SettingRepository
	userRepository repositories.UserRepository
}

func NewSettingService(repo *repositories.SettingRepository, userRepository repositories.UserRepository) *SettingService {
	return &SettingService{repo: repo, userRepository: userRepository}
}

func (s *SettingService) GetSettings() (map[string]string, error) {
	settings, err := s.repo.GetSettings()
	if err != nil {
		return nil, err
	}

	settingsMap := make(map[string]string)
	for _, setting := range settings {
		settingsMap[setting.Key] = setting.Value
	}
	return settingsMap, nil
}

func (s *SettingService) GetPublicSettings() (map[string]string, error) {
	return s.GetSettings()
}

func (s *SettingService) UpdateSettings(settingsMap map[string]string) error {
	var settings []models.Setting
	for key, value := range settingsMap {
		settings = append(settings, models.Setting{Key: key, Value: value})
	}
	return s.repo.UpsertSettings(settings)
}