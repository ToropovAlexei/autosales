package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/gin-gonic/gin"
)

type SettingService struct {
	repo            repositories.SettingRepository
	userRepository  repositories.UserRepository
	auditLogService AuditLogService
}

func NewSettingService(repo repositories.SettingRepository, userRepository repositories.UserRepository, auditLogService AuditLogService) *SettingService {
	return &SettingService{repo: repo, userRepository: userRepository, auditLogService: auditLogService}
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
	allSettings, err := s.GetSettings()
	if err != nil {
		return nil, err
	}

	publicSettings := make(map[string]string)
	publicKeys := []string{
		"referral_program_enabled",
		"referral_percentage",
		"GATEWAY_BONUS_mock_provider",
		"GATEWAY_BONUS_platform_card",
		"GATEWAY_BONUS_platform_sbp",
		"support_message",
		"welcome_message",
	}

	for _, key := range publicKeys {
		if value, ok := allSettings[key]; ok {
			publicSettings[key] = value
		}
	}

	return publicSettings, nil
}

func (s *SettingService) UpdateSettings(ctx *gin.Context, settingsMap map[string]string) error {
	before, err := s.GetSettings()
	if err != nil {
		// Log the error but proceed, as we might be creating settings for the first time.
		// In a real-world scenario, you might want more sophisticated error handling.
	}

	var settings []models.Setting
	for key, value := range settingsMap {
		settings = append(settings, models.Setting{Key: key, Value: value})
	}

	if err := s.repo.UpsertSettings(settings); err != nil {
		return err
	}

	s.auditLogService.Log(ctx, "SETTINGS_UPDATE", "GlobalSettings", 0, map[string]interface{}{"before": before, "after": settingsMap})

	return nil
}
