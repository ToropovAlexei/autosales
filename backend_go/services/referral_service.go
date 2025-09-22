package services

import (
	"frbktg/backend_go/apperrors"
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
	UpdateReferralBotStatus(botID uint, sellerID uint, isActive bool) (*models.ReferralBot, error)
	SetPrimary(botID uint, sellerID uint) error
	DeleteReferralBot(botID uint, sellerID uint) error
}

type referralService struct {
	userRepo     repositories.UserRepository
	botUserRepo  repositories.BotUserRepository
	referralRepo repositories.ReferralRepository
	transRepo    repositories.TransactionRepository
}

func NewReferralService(userRepo repositories.UserRepository, botUserRepo repositories.BotUserRepository, referralRepo repositories.ReferralRepository, transRepo repositories.TransactionRepository) ReferralService {
	return &referralService{
		userRepo:     userRepo,
		botUserRepo:  botUserRepo,
		referralRepo: referralRepo,
		transRepo:    transRepo,
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
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "User", ID: uint(ownerTelegramID)}
	}

	// Проверяем лимит ботов
	count, err := s.referralRepo.CountByOwnerID(owner.ID)
	if err != nil {
		return nil, apperrors.New(500, "Failed to count user bots", err)
	}
	if count >= 3 {
		return nil, apperrors.ErrBotLimitExceeded
	}

	// We assume seller is validated in the handler via auth context

	_, err = s.referralRepo.FindReferralBotByToken(botToken)
	if err == nil {
		return nil, &apperrors.ErrAlreadyExists{Base: apperrors.New(409, "", nil), Resource: "ReferralBot", Field: "token", Value: botToken}
	}

	dbBot := &models.ReferralBot{
		OwnerID:  owner.ID,
		SellerID: sellerID,
		BotToken: botToken,
	}

	if err := s.referralRepo.CreateReferralBot(dbBot); err != nil {
		return nil, apperrors.New(500, "Failed to create referral bot", err)
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

func (s *referralService) GetAdminInfoForSeller(sellerID uint) ([]models.ReferralBotAdminInfo, error) {
	return s.referralRepo.GetAdminInfoForSeller(sellerID)
}

func (s *referralService) UpdateReferralBotStatus(botID uint, sellerID uint, isActive bool) (*models.ReferralBot, error) {
	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "ReferralBot", ID: botID}
	}

	if bot.SellerID != sellerID {
		return nil, apperrors.ErrForbidden
	}

	bot.IsActive = isActive
	if err := s.referralRepo.UpdateReferralBot(bot); err != nil {
		return nil, apperrors.New(500, "Failed to update referral bot", err)
	}

	return bot, nil
}

func (s *referralService) SetPrimary(botID uint, sellerID uint) error {
	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "ReferralBot", ID: botID}
	}

	if bot.SellerID != sellerID {
		return apperrors.ErrForbidden
	}

	return s.referralRepo.SetPrimary(sellerID, botID)
}

func (s *referralService) DeleteReferralBot(botID uint, sellerID uint) error {
	bot, err := s.referralRepo.GetReferralBotByID(botID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "ReferralBot", ID: botID}
	}

	if bot.SellerID != sellerID {
		return apperrors.ErrForbidden
	}

	return s.referralRepo.DeleteReferralBot(bot)
}
