package services

import (
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"strconv"
	"strings"

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
		"new_user_welcome_message",
		"returning_user_welcome_message",
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

	// Validate settings
	for key, value := range settingsMap {
		switch {
		case key == "GLOBAL_PRICE_MARKUP":
			if err := validateFloat(value, 0, 30); err != nil {
				return apperrors.New(400, fmt.Sprintf("Invalid value for %s: %v", key, err), nil)
			}
		case strings.HasPrefix(key, "GATEWAY_COMMISSION_"):
			if err := validateFloat(value, 0, 25); err != nil {
				return apperrors.New(400, fmt.Sprintf("Invalid value for %s: %v", key, err), nil)
			}
		case strings.HasPrefix(key, "GATEWAY_BONUS_"):
			if err := validateFloat(value, 0, 25.8); err != nil {
				return apperrors.New(400, fmt.Sprintf("Invalid value for %s: %v", key, err), nil)
			}
		}
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

func validateFloat(value string, min, max float64) error {
	f, err := strconv.ParseFloat(value, 64)
	if err != nil {
		return fmt.Errorf("must be a valid number")
	}
	if f < min || f > max {
		return fmt.Errorf("must be between %.2f and %.2f", min, max)
	}
	return nil
}
