package services

import (
	"encoding/base64"
	"errors"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"

	"golang.org/x/crypto/bcrypt"
)

type UserService interface {
	GetMe(user models.User) *models.UserResponse
	GetMeByEmail(email string) (*models.User, error)
	GetUsers(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.User], error)
	CreateUser(ctx *gin.Context, email, password string, roleID uint) (*models.CreateUserResponse, error)
	UpdateReferralSettings(ctx *gin.Context, user *models.User, enabled bool, percentage float64) error
	RegisterBotUser(telegramID int64, botName string) (*models.BotUser, float64, bool, bool, error)
	GetBotUser(id uint) (*models.BotUser, float64, error)
	GetBotUserByTelegramID(telegramID int64, botName string) (*models.BotUser, float64, error)
	ToggleBlockUser(ctx *gin.Context, telegramID int64) error
	GetUserBalance(telegramID int64) (float64, error)
	GetUserTransactions(telegramID int64) ([]models.Transaction, error)
	GetUserSubscriptionsByTelegramID(telegramID int64) ([]models.UserSubscription, error)
	GetUserOrdersByTelegramID(telegramID int64) ([]models.Order, error)
	UpdateUserCaptchaStatus(id uint, hasPassed bool) error
	UpdateUserCaptchaStatusByTelegramID(telegramID int64, hasPassed bool) error
}

type userService struct {
	userRepo             repositories.UserRepository
	botUserRepo          repositories.BotUserRepository
	userSubscriptionRepo repositories.UserSubscriptionRepository
	orderRepo            repositories.OrderRepository
	auditLogService      AuditLogService
	twoFAService         TwoFAService
}

func NewUserService(userRepo repositories.UserRepository, botUserRepo repositories.BotUserRepository, userSubscriptionRepo repositories.UserSubscriptionRepository, orderRepo repositories.OrderRepository, auditLogService AuditLogService, twoFAService TwoFAService) UserService {
	return &userService{
		userRepo:             userRepo,
		botUserRepo:          botUserRepo,
		userSubscriptionRepo: userSubscriptionRepo,
		orderRepo:            orderRepo,
		auditLogService:      auditLogService,
		twoFAService:         twoFAService,
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
		ReferralProgramEnabled: user.ReferralProgramEnabled,
		ReferralPercentage:     user.ReferralPercentage,
	}
}

func (s *userService) GetMeByEmail(email string) (*models.User, error) {
	return s.userRepo.FindByEmail(email)
}

func (s *userService) GetUsers(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.User], error) {
	return s.userRepo.GetUsers(page, filters)
}

func (s *userService) UpdateReferralSettings(ctx *gin.Context, user *models.User, enabled bool, percentage float64) error {
	before := *user
	if err := s.userRepo.UpdateReferralSettings(user, enabled, percentage); err != nil {
		return err
	}
	after, _ := s.userRepo.FindByID(user.ID)
	s.auditLogService.Log(ctx, "USER_UPDATE_REFERRAL_SETTINGS", "User", user.ID, map[string]interface{}{"before": before, "after": after})
	return nil
}

func (s *userService) RegisterBotUser(telegramID int64, botName string) (*models.BotUser, float64, bool, bool, error) {
	existingUser, err := s.botUserRepo.FindByTelegramID(telegramID)

	if err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return nil, 0, false, false, apperrors.New(500, "Failed to find bot user", err)
	}

	// If user exists
	if existingUser != nil {
		// If user is blocked, do nothing and return their status
		if existingUser.IsBlocked {
			return existingUser, 0, false, existingUser.HasPassedCaptcha, nil
		}

		// Otherwise, update their last seen info
		existingUser.LastSeenWithBot = botName
		existingUser.LastSeenAt = time.Now()

		balance, err := s.botUserRepo.GetUserBalance(existingUser.ID)
		if err != nil {
			return nil, 0, false, false, apperrors.New(500, "Failed to get user balance", err)
		}
		if err := s.botUserRepo.Update(existingUser); err != nil {
			return nil, 0, false, false, apperrors.New(500, "Failed to update bot user", err)
		}
		return existingUser, balance, false, existingUser.HasPassedCaptcha, nil
	}

	// If user does not exist, create a new one
	newUser := &models.BotUser{
		TelegramID:        telegramID,
		HasPassedCaptcha:  false,
		RegisteredWithBot: botName,
		LastSeenWithBot:   botName,
		LastSeenAt:        time.Now(),
	}
	if err := s.botUserRepo.Create(newUser); err != nil {
		return nil, 0, false, false, apperrors.New(500, "Failed to create bot user", err)
	}

	return newUser, 0, true, false, nil
}

func (s *userService) ToggleBlockUser(ctx *gin.Context, telegramID int64) error {
	user, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(telegramID)}
	}

	before := *user
	user.IsBlocked = !user.IsBlocked

	if err := s.botUserRepo.Update(user); err != nil {
		return err
	}

	s.auditLogService.Log(ctx, "USER_TOGGLE_BLOCK", "BotUser", user.ID, map[string]interface{}{"before": before, "after": user})

	return nil
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
	user.LastSeenAt = time.Now()
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

func (s *userService) CreateUser(ctx *gin.Context, email, password string, roleID uint) (*models.CreateUserResponse, error) {
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, err
	}

	secret, err := s.twoFAService.GenerateSecret(email)
	if err != nil {
		return nil, err
	}

	encryptedSecret, err := s.twoFAService.EncryptSecret(secret)
	if err != nil {
		return nil, err
	}

	user := &models.User{
		Email:          email,
		HashedPassword: string(hashedPassword),
		IsActive:       true,
		TwoFASecret:    &encryptedSecret,
	}

	if err := s.userRepo.Create(user); err != nil {
		return nil, err
	}

	if err := s.userRepo.SetUserRole(user.ID, roleID); err != nil {
		// TODO: maybe delete the user if role assignment fails?
		return nil, err
	}

	qrCode, err := s.twoFAService.GenerateQRCode(email, secret)
	if err != nil {
		return nil, err
	}

	s.auditLogService.Log(ctx, "USER_CREATE", "User", user.ID, map[string]interface{}{"after": user})

	return &models.CreateUserResponse{
		User: models.UserResponse{
			ID:       user.ID,
			Email:    user.Email,
			IsActive: user.IsActive,
		},
		TwoFASecret: secret,
		QRCode:    base64.StdEncoding.EncodeToString(qrCode),
	}, nil
}
