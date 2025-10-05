package services

import (
	"errors"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"gorm.io/gorm"
)

type UserService interface {
	GetMe(user models.User) *models.UserResponse
	UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error
	RegisterBotUser(telegramID int64, botName string) (*models.BotUser, float64, bool, bool, error)
	GetBotUser(id uint) (*models.BotUser, float64, error)
	GetBotUserByTelegramID(telegramID int64, botName string) (*models.BotUser, float64, error)
	GetUserBalance(telegramID int64) (float64, error)
	GetUserTransactions(telegramID int64) ([]models.Transaction, error)
	GetUserSubscriptionsByTelegramID(telegramID int64) ([]models.UserSubscription, error)
	GetUserOrdersByTelegramID(telegramID int64) ([]models.Order, error)
	UpdateUserCaptchaStatus(id uint, hasPassed bool) error
	UpdateUserCaptchaStatusByTelegramID(telegramID int64, hasPassed bool) error
	GetSellerSettings() (*models.User, error)
}

type userService struct {
	userRepo             repositories.UserRepository
	botUserRepo          repositories.BotUserRepository
	userSubscriptionRepo repositories.UserSubscriptionRepository
	orderRepo            repositories.OrderRepository
}

func NewUserService(userRepo repositories.UserRepository, botUserRepo repositories.BotUserRepository, userSubscriptionRepo repositories.UserSubscriptionRepository, orderRepo repositories.OrderRepository) UserService {
	return &userService{
		userRepo:             userRepo,
		botUserRepo:          botUserRepo,
		userSubscriptionRepo: userSubscriptionRepo,
		orderRepo:            orderRepo,
	}
}

func (s *userService) GetUserSubscriptionsByTelegramID(telegramID int64) ([]models.UserSubscription, error) {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(telegramID)}
	}
	return s.userSubscriptionRepo.FindSubscriptionsByBotUserID(user.ID)
}

func (s *userService) GetUserOrdersByTelegramID(telegramID int64) ([]models.Order, error) {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(telegramID)}
	}
	return s.orderRepo.FindOrdersByBotUserID(user.ID)
}

func (s *userService) GetMe(user models.User) *models.UserResponse {
	return &models.UserResponse{
		ID:                     user.ID,
		Email:                  user.Email,
		IsActive:               user.IsActive,
		Role:                   user.Role,
		ReferralProgramEnabled: user.ReferralProgramEnabled,
		ReferralPercentage:     user.ReferralPercentage,
	}
}

func (s *userService) UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error {
	return s.userRepo.UpdateReferralSettings(user, enabled, percentage)
}

func (s *userService) RegisterBotUser(telegramID int64, botName string) (*models.BotUser, float64, bool, bool, error) {
	existingUser, err := s.botUserRepo.FindByTelegramID(telegramID)

	if err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return nil, 0, false, false, apperrors.New(500, "Failed to find bot user", err)
	}

	if existingUser != nil {
		
		existingUser.LastSeenWithBot = botName

		if !existingUser.IsDeleted {
			balance, err := s.botUserRepo.GetUserBalance(existingUser.ID)
			if err != nil {
				return nil, 0, false, false, apperrors.New(500, "Failed to get user balance", err)
			}
			if err := s.botUserRepo.Update(existingUser); err != nil {
				return nil, 0, false, false, apperrors.New(500, "Failed to update bot user", err)
			}
			return existingUser, balance, false, existingUser.HasPassedCaptcha, nil
		}

		existingUser.IsDeleted = false
		existingUser.HasPassedCaptcha = false
		if err := s.botUserRepo.Update(existingUser); err != nil {
			return nil, 0, false, false, apperrors.New(500, "Failed to update bot user", err)
		}
		return existingUser, 0, true, false, nil
	}

	newUser := &models.BotUser{
		TelegramID:       telegramID,
		HasPassedCaptcha: false,
		RegisteredWithBot: botName,
		LastSeenWithBot:   botName,
	}
	if err := s.botUserRepo.Create(newUser); err != nil {
		return nil, 0, false, false, apperrors.New(500, "Failed to create bot user", err)
	}

	return newUser, 0, true, false, nil
}

func (s *userService) GetBotUser(id uint) (*models.BotUser, float64, error) {
	user, err := s.botUserRepo.FindByID(id)
	if err != nil {
		return nil, 0, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: id}
	}

	balance, err := s.botUserRepo.GetUserBalance(user.ID)
	if err != nil {
		return nil, 0, apperrors.New(500, "Failed to get user balance", err)
	}

	return user, balance, nil
}

func (s *userService) GetBotUserByTelegramID(telegramID int64, botName string) (*models.BotUser, float64, error) {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return nil, 0, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(telegramID)}
	}

	user.LastSeenWithBot = botName
	if err := s.botUserRepo.Update(user); err != nil {
		return nil, 0, apperrors.New(500, "Failed to update bot user", err)
	}

	balance, err := s.botUserRepo.GetUserBalance(user.ID)
	if err != nil {
		return nil, 0, apperrors.New(500, "Failed to get user balance", err)
	}

	return user, balance, nil
}

func (s *userService) GetUserBalance(telegramID int64) (float64, error) {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return 0, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(telegramID)}
	}
	return s.botUserRepo.GetUserBalance(user.ID)
}

func (s *userService) GetUserTransactions(telegramID int64) ([]models.Transaction, error) {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(telegramID)}
	}
	return s.botUserRepo.GetUserTransactions(user.ID)
}

func (s *userService) UpdateUserCaptchaStatus(id uint, hasPassed bool) error {
	user, err := s.botUserRepo.FindByID(id)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: id}
	}
	return s.botUserRepo.UpdateCaptchaStatus(user, hasPassed)
}

func (s *userService) UpdateUserCaptchaStatusByTelegramID(telegramID int64, hasPassed bool) error {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(telegramID)}
	}
	return s.botUserRepo.UpdateCaptchaStatus(user, hasPassed)
}

func (s *userService) GetSellerSettings() (*models.User, error) {
	seller, err := s.userRepo.FindSellerSettings()
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "SellerSettings"}
	}
	return seller, nil
}