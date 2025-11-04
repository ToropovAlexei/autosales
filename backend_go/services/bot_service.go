package services

import (
	"encoding/json"
	"errors"
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"net/http"
	"strconv"

	"github.com/jackc/pgx/v5/pgconn"
)

// Helper to get bot info from Telegram API
func getBotInfo(token string) (string, error) {
	resp, err := http.Get(fmt.Sprintf("https://api.telegram.org/bot%s/getMe", token))
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("telegram API returned non-200 status: %d", resp.StatusCode)
	}

	var result struct {
		OK     bool `json:"ok"`
		Result struct {
			Username string `json:"username"`
		} `json:"result"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	if !result.OK || result.Result.Username == "" {
		return "", fmt.Errorf("invalid bot token or username not found")
	}

	return result.Result.Username, nil
}

type BotService interface {
	FindBotByName(name string) (*models.Bot, error)
	FindBotByID(id uint) (*models.Bot, error)
	CreateBot(bot *models.Bot) error
	CreateReferralBot(ownerTelegramID int64, botToken string) (*models.Bot, error)
	GetAllBots(botType string) ([]models.BotResponse, error)
	GetMainBots() ([]models.BotResponse, error)
}

type botService struct {
	botRepo        repositories.BotRepository
	botUserRepo    repositories.BotUserRepository
	settingService SettingService
}

func NewBotService(botRepo repositories.BotRepository, botUserRepo repositories.BotUserRepository, settingService SettingService) BotService {
	return &botService{botRepo: botRepo, botUserRepo: botUserRepo, settingService: settingService}
}

func (s *botService) FindBotByName(name string) (*models.Bot, error) {
	return s.botRepo.FindByName(name)
}

func (s *botService) FindBotByID(id uint) (*models.Bot, error) {
	return s.botRepo.FindByID(id)
}

func (s *botService) CreateBot(bot *models.Bot) error {
	return s.botRepo.Create(bot)
}

func (s *botService) CreateReferralBot(ownerTelegramID int64, botToken string) (*models.Bot, error) {
	owner, err := s.botUserRepo.FindByTelegramID(ownerTelegramID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(http.StatusNotFound, "", err), Resource: "User", ID: uint(ownerTelegramID)}
	}

	// Check bot limit
	count, err := s.botRepo.CountByOwnerID(owner.ID)
	if err != nil {
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to count user bots", err)
	}
	if count >= 3 {
		return nil, apperrors.ErrBotLimitExceeded
	}

	botUsername, err := getBotInfo(botToken)
	if err != nil {
		return nil, &apperrors.ErrValidation{Message: "Invalid bot token: " + err.Error()}
	}

	bot, err := s.botRepo.FindByToken(botToken)
	if err != nil {
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to check for existing bot", err)
	}
	if bot != nil {
		return nil, &apperrors.ErrAlreadyExists{Base: apperrors.New(http.StatusConflict, "", nil), Resource: "Bot", Field: "token", Value: botToken}
	}

	settings, err := s.settingService.GetSettings()
	if err != nil {
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to get settings", err)
	}
	defaultPercentage, _ := strconv.ParseFloat(settings["referral_percentage"], 64)

	dbBot := &models.Bot{
		OwnerID:            &owner.ID,
		Token:              botToken,
		Username:           botUsername,
		Type:               "referral",
		ReferralPercentage: defaultPercentage,
	}

	if err := s.botRepo.Create(dbBot); err != nil {
		var pgErr *pgconn.PgError
		if errors.As(err, &pgErr) && pgErr.Code == "23505" { // unique_violation
			return nil, &apperrors.ErrAlreadyExists{Base: apperrors.New(http.StatusConflict, "", nil), Resource: "Bot", Field: "token or username", Value: botToken}
		}
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to create referral bot", err)
	}

	return dbBot, nil
}

func (s *botService) GetAllBots(botType string) ([]models.BotResponse, error) {
	bots, err := s.botRepo.GetAll(botType)
	if err != nil {
		return nil, err
	}

	var botResponses []models.BotResponse
	for _, bot := range bots {
		botResponses = append(botResponses, s.toBotResponse(bot))
	}

	return botResponses, nil
}

func (s *botService) GetMainBots() ([]models.BotResponse, error) {
	bots, err := s.botRepo.GetAll("main")
	if err != nil {
		return nil, err
	}

	var botResponses []models.BotResponse
	for _, bot := range bots {
		botResponses = append(botResponses, s.toBotResponse(bot))
	}

	return botResponses, nil
}

func (s *botService) toBotResponse(bot models.Bot) models.BotResponse {
	return models.BotResponse{
		ID:        bot.ID,
		Token:     bot.Token,
		Username:  bot.Username,
		IsPrimary: bot.IsPrimary,
		IsActive:  bot.IsActive,
		OwnerID:   bot.OwnerID,
	}
}