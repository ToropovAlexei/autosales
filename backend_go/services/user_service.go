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
	RegisterBotUser(telegramID int64) (*models.BotUser, float64, bool, bool, error)
	GetBotUser(id uint) (*models.BotUser, float64, error)
	GetUserBalance(telegramID int64) (float64, error)
	GetUserTransactions(telegramID int64) ([]models.Transaction, error)
	UpdateUserCaptchaStatus(id uint, hasPassed bool) error
	GetSellerSettings() (*models.User, error)
}

type userService struct {
	userRepo    repositories.UserRepository
	botUserRepo repositories.BotUserRepository
}

func NewUserService(userRepo repositories.UserRepository, botUserRepo repositories.BotUserRepository) UserService {
	return &userService{userRepo: userRepo, botUserRepo: botUserRepo}
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

func (s *userService) RegisterBotUser(telegramID int64) (*models.BotUser, float64, bool, bool, error) {
	existingUser, err := s.botUserRepo.FindByTelegramID(telegramID)

	if err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return nil, 0, false, false, apperrors.New(500, "Failed to find bot user", err)
	}

	if existingUser != nil {
		if !existingUser.IsDeleted {
			balance, err := s.botUserRepo.GetUserBalance(existingUser.ID)
			if err != nil {
				return nil, 0, false, false, apperrors.New(500, "Failed to get user balance", err)
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

	newUser := &models.BotUser{TelegramID: telegramID, HasPassedCaptcha: false}
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

func (s *userService) GetSellerSettings() (*models.User, error) {
	seller, err := s.userRepo.FindSellerSettings()
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "SellerSettings"}
	}
	return seller, nil
}