package services

import (
	"errors"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"net/http"
	"strconv"
	"time"

	"github.com/jackc/pgx/v5/pgconn"
	"gorm.io/gorm"
)



type ReferralService interface {
	ProcessReferral(tx *gorm.DB, referralBotToken *string, order models.Order, orderAmount float64) error
	CreateReferralBot(ownerTelegramID int64, botToken string) (*models.ReferralBot, error)
	GetAllReferralBots() ([]models.ReferralBotResponse, error)
	GetReferralBotByID(botID uint) (*models.ReferralBot, error)
	GetReferralBotsByTelegramID(telegramID int64) ([]models.ReferralBotAdminInfo, error)
	GetAllAdminInfo() ([]models.ReferralBotAdminInfo, error)
	UpdateReferralBotStatus(botID uint, ownerID uint, isActive bool) (*models.ReferralBot, error)
	SetPrimary(botID uint, ownerID uint) error
	DeleteReferralBot(botID uint, ownerID uint) error
	ServiceSetPrimary(botID uint, telegramID int64) error
	ServiceDeleteReferralBot(botID uint, telegramID int64) error
}

type referralService struct {
	userRepo       repositories.UserRepository
	botUserRepo    repositories.BotUserRepository
	referralRepo   repositories.ReferralRepository
	transRepo      repositories.TransactionRepository
	settingService SettingService
}

func NewReferralService(userRepo repositories.UserRepository, botUserRepo repositories.BotUserRepository, referralRepo repositories.ReferralRepository, transRepo repositories.TransactionRepository, settingService SettingService) ReferralService {
	return &referralService{
		userRepo:       userRepo,
		botUserRepo:    botUserRepo,
		referralRepo:   referralRepo,
		transRepo:      transRepo,
		settingService: settingService,
	}
}

func (s *referralService) GetAllAdminInfo() ([]models.ReferralBotAdminInfo, error) {
	bots, err := s.referralRepo.GetAllAdminInfo()
	if err != nil {
		return nil, apperrors.New(500, "Failed to get all referral bots admin info", err)
	}
	return bots, nil
}

func (s *referralService) GetReferralBotByID(botID uint) (*models.ReferralBot, error) {
	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "ReferralBot", ID: botID}
	}
	return bot, nil
}


func (s *referralService) ProcessReferral(tx *gorm.DB, referralBotToken *string, order models.Order, orderAmount float64) error {
	if referralBotToken == nil {
		return nil
	}

	settings, err := s.settingService.GetSettings()
	if err != nil {
		// Log the error but don't fail the transaction, as referral is non-critical
		return nil
	}

	if enabled, _ := strconv.ParseBool(settings["referral_program_enabled"]); !enabled {
		return nil
	}

	percentage, _ := strconv.ParseFloat(settings["referral_percentage"], 64)
	if percentage <= 0 {
		return nil
	}

	refBot, err := s.referralRepo.WithTx(tx).FindByBotToken(*referralBotToken)
	if err != nil || !refBot.IsActive {
		// Bot not found or not active, just ignore
		return nil
	}

	refShare := orderAmount * (percentage / percentDenominator)
	refTransaction := &models.RefTransaction{
		RefOwnerID: refBot.OwnerID,
		OrderID:    order.ID,
		Amount:     orderAmount,
		RefShare:   refShare,
		CreatedAt:  time.Now().UTC(),
	}
	return s.transRepo.WithTx(tx).CreateRefTransaction(refTransaction)
}

func (s *referralService) CreateReferralBot(ownerTelegramID int64, botToken string) (*models.ReferralBot, error) {
	owner, err := s.botUserRepo.FindByTelegramID(ownerTelegramID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(http.StatusNotFound, "", err), Resource: "User", ID: uint(ownerTelegramID)}
	}

	// Проверяем лимит ботов
	count, err := s.referralRepo.CountByOwnerID(owner.ID)
	if err != nil {
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to count user bots", err)
	}
	if count >= 3 {
		return nil, apperrors.ErrBotLimitExceeded
	}

	_, err = s.referralRepo.FindReferralBotByToken(botToken)
	// We expect a "not found" error here. If any other error occurs, or if no error occurs (meaning bot was found), we fail.
	if !errors.Is(err, gorm.ErrRecordNotFound) {
		if err == nil { // Bot was found
			return nil, &apperrors.ErrAlreadyExists{Base: apperrors.New(http.StatusConflict, "", nil), Resource: "ReferralBot", Field: "token", Value: botToken}
		}
		// Another unexpected error
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to check for existing bot", err)
	}

	dbBot := &models.ReferralBot{
		OwnerID:  owner.ID,
		BotToken: botToken,
	}

	if err := s.referralRepo.CreateReferralBot(dbBot); err != nil {
		var pgErr *pgconn.PgError
		if errors.As(err, &pgErr) && pgErr.Code == "23505" { // unique_violation
			return nil, &apperrors.ErrAlreadyExists{Base: apperrors.New(http.StatusConflict, "", nil), Resource: "ReferralBot", Field: "token", Value: botToken}
		}
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to create referral bot", err)
	}

	return dbBot, nil
}

func (s *referralService) GetAllReferralBots() ([]models.ReferralBotResponse, error) {
	bots, err := s.referralRepo.GetAllReferralBots()
	if err != nil {
		return nil, apperrors.New(500, "Failed to get all referral bots", err)
	}

	var response []models.ReferralBotResponse
	for _, b := range bots {
		response = append(response, models.ReferralBotResponse(b))
	}

	return response, nil
}

func (s *referralService) GetReferralBotsByTelegramID(telegramID int64) ([]models.ReferralBotAdminInfo, error) {
	owner, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "User", ID: uint(telegramID)}
	}

	return s.referralRepo.GetAdminInfoForOwner(owner.ID)
}

func (s *referralService) UpdateReferralBotStatus(botID uint, ownerID uint, isActive bool) (*models.ReferralBot, error) {
	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "ReferralBot", ID: botID}
	}

	if bot.OwnerID != ownerID {
		return nil, apperrors.ErrForbidden
	}

	bot.IsActive = isActive
	if err := s.referralRepo.UpdateReferralBot(bot); err != nil {
		return nil, apperrors.New(500, "Failed to update referral bot", err)
	}

	return bot, nil
}

func (s *referralService) SetPrimary(botID uint, ownerID uint) error {
	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "ReferralBot", ID: botID}
	}

	if bot.OwnerID != ownerID {
		return apperrors.ErrForbidden
	}

	return s.referralRepo.SetPrimary(ownerID, botID)
}

func (s *referralService) DeleteReferralBot(botID uint, ownerID uint) error {
	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "ReferralBot", ID: botID}
	}

	if bot.OwnerID != ownerID {
		return apperrors.ErrForbidden
	}

	return s.referralRepo.DeleteReferralBot(bot)
}

func (s *referralService) ServiceSetPrimary(botID uint, telegramID int64) error {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(http.StatusNotFound, "", err), Resource: "User", ID: uint(telegramID)}
	}

	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(http.StatusNotFound, "", err), Resource: "ReferralBot", ID: botID}
	}

	if bot.OwnerID != user.ID {
		return apperrors.ErrForbidden
	}

	return s.referralRepo.SetPrimary(user.ID, botID)
}

func (s *referralService) ServiceDeleteReferralBot(botID uint, telegramID int64) error {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(http.StatusNotFound, "", err), Resource: "User", ID: uint(telegramID)}
	}

	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(http.StatusNotFound, "", err), Resource: "ReferralBot", ID: botID}
	}

	if bot.OwnerID != user.ID {
		return apperrors.ErrForbidden
	}

	return s.referralRepo.DeleteReferralBot(bot)
}
