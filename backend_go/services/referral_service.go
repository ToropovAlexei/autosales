package services

import (
	"errors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"

	"gorm.io/gorm"
)

type ReferralService interface {
	ProcessReferral(tx *gorm.DB, referralBotToken *string, order models.Order, orderAmount float64) error
	CreateReferralBot(ownerTelegramID int64, sellerID uint, botToken string) (*models.ReferralBot, error)
	GetAllReferralBots() ([]models.ReferralBotResponse, error)
	GetAdminInfoForSeller(sellerID uint) ([]models.ReferralBotAdminInfo, error)
	ToggleReferralBotStatus(botID uint, sellerID uint) (*models.ReferralBot, error)
}

type referralService struct {
	userRepo    repositories.UserRepository
	botUserRepo repositories.BotUserRepository
	referralRepo repositories.ReferralRepository
	transRepo   repositories.TransactionRepository
}

func NewReferralService(userRepo repositories.UserRepository, botUserRepo repositories.BotUserRepository, referralRepo repositories.ReferralRepository, transRepo repositories.TransactionRepository) ReferralService {
	return &referralService{
		userRepo:    userRepo,
		botUserRepo: botUserRepo,
		referralRepo: referralRepo,
		transRepo:   transRepo,
	}
}

func (s *referralService) ProcessReferral(tx *gorm.DB, referralBotToken *string, order models.Order, orderAmount float64) error {
	if referralBotToken == nil {
		return nil
	}

	refBot, err := s.referralRepo.WithTx(tx).FindByBotToken(*referralBotToken)
	if err != nil || !refBot.IsActive {
		// Bot not found or not active, just ignore
		return nil
	}

	seller, err := s.userRepo.WithTx(tx).FindByID(refBot.SellerID)
	if err != nil {
		// Seller not found, ignore
		return nil
	}

	if seller.ReferralProgramEnabled && seller.ReferralPercentage > 0 {
		refShare := orderAmount * (seller.ReferralPercentage / percentDenominator)
		refTransaction := &models.RefTransaction{
			RefOwnerID: refBot.OwnerID,
			SellerID:   seller.ID,
			OrderID:    order.ID,
			Amount:     orderAmount,
			RefShare:   refShare,
			CreatedAt:  time.Now().UTC(),
		}
		return s.transRepo.WithTx(tx).CreateRefTransaction(refTransaction)
	}

	return nil
}

func (s *referralService) CreateReferralBot(ownerTelegramID int64, sellerID uint, botToken string) (*models.ReferralBot, error) {
	owner, err := s.botUserRepo.FindByTelegramID(ownerTelegramID)
	if err != nil {
		return nil, errors.New("referral owner (user) not found")
	}

	// We assume seller is validated in the handler via auth context

	_, err = s.referralRepo.FindReferralBotByToken(botToken)
	if err == nil {
		return nil, errors.New("bot with this token already exists")
	}

	dbBot := &models.ReferralBot{
		OwnerID:  owner.ID,
		SellerID: sellerID,
		BotToken: botToken,
	}

	if err := s.referralRepo.CreateReferralBot(dbBot); err != nil {
		return nil, err
	}

	return dbBot, nil
}

func (s *referralService) GetAllReferralBots() ([]models.ReferralBotResponse, error) {
	bots, err := s.referralRepo.GetAllReferralBots()
	if err != nil {
		return nil, err
	}

	var response []models.ReferralBotResponse
	for _, b := range bots {
		response = append(response, models.ReferralBotResponse(b))
	}

	return response, nil
}

func (s *referralService) GetAdminInfoForSeller(sellerID uint) ([]models.ReferralBotAdminInfo, error) {
	return s.referralRepo.GetAdminInfoForSeller(sellerID)
}

func (s *referralService) ToggleReferralBotStatus(botID uint, sellerID uint) (*models.ReferralBot, error) {
	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return nil, errors.New("referral bot not found")
	}

	if bot.SellerID != sellerID {
		return nil, errors.New("you are not the owner of this referral bot")
	}

	bot.IsActive = !bot.IsActive
	if err := s.referralRepo.UpdateReferralBot(bot); err != nil {
		return nil, err
	}

	return bot, nil
}
