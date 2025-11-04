package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"strconv"
	"time"

	"gorm.io/gorm"
)

type ReferralService interface {
	ProcessReferral(tx *gorm.DB, botID uint, order models.Order, orderAmount float64) error
	GetReferralStats(telegramID int64) (map[uint]models.ReferralBotStats, error)
}

type referralService struct {
	botRepo        repositories.BotRepository
	botUserRepo    repositories.BotUserRepository
	statsRepo      repositories.StatsRepository
	transRepo      repositories.TransactionRepository
	settingService SettingService
}

func NewReferralService(botRepo repositories.BotRepository, botUserRepo repositories.BotUserRepository, statsRepo repositories.StatsRepository, transRepo repositories.TransactionRepository, settingService SettingService) ReferralService {
	return &referralService{
		botRepo:        botRepo,
		botUserRepo:    botUserRepo,
		statsRepo:      statsRepo,
		transRepo:      transRepo,
		settingService: settingService,
	}
}

func (s *referralService) ProcessReferral(tx *gorm.DB, botID uint, order models.Order, orderAmount float64) error {
	if botID == 0 {
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

	bot, err := s.botRepo.WithTx(tx).FindByID(botID)
	if err != nil || bot == nil || !bot.IsActive || bot.Type != "referral" || bot.OwnerID == nil {
		// Bot not found, not active, not a referral bot, or has no owner, just ignore
		return nil
	}

	percentage := bot.ReferralPercentage
	if percentage <= 0 {
		return nil
	}

	refShare := orderAmount * (percentage / percentDenominator)
	refTransaction := &models.RefTransaction{
		RefOwnerID: *bot.OwnerID,
		OrderID:    order.ID,
		Amount:     orderAmount,
		RefShare:   refShare,
		CreatedAt:  time.Now().UTC(),
	}
	return s.transRepo.WithTx(tx).CreateRefTransaction(refTransaction)
}

func (s *referralService) GetReferralStats(telegramID int64) (map[uint]models.ReferralBotStats, error) {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return nil, err
	}

	return s.statsRepo.GetReferralStats(user.ID)
}
