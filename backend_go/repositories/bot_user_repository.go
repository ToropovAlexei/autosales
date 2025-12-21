package repositories

import (
	"errors"
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type BotUserRepository interface {
	WithTx(tx *gorm.DB) BotUserRepository
	FindByTelegramID(telegramID int64) (*models.BotUser, error)
	FindByID(id uint) (*models.BotUser, error)
	Create(user *models.BotUser) error
	Update(user *models.BotUser) error
	UpdateCaptchaStatus(user *models.BotUser, hasPassed bool) error
	UpdateBalance(userID uint, amount float64) error
	GetUserBalance(userID uint) (float64, error)
	GetUserTransactions(userID uint) ([]models.Transaction, error)
	UpdateBotUserStatus(telegramID int64, updates map[string]interface{}) error
}

type gormBotUserRepository struct {
	db *gorm.DB
}

func NewBotUserRepository(db *gorm.DB) BotUserRepository {
	return &gormBotUserRepository{db: db}
}

func (r *gormBotUserRepository) WithTx(tx *gorm.DB) BotUserRepository {
	return &gormBotUserRepository{db: tx}
}

func (r *gormBotUserRepository) FindByTelegramID(telegramID int64) (*models.BotUser, error) {
	var user models.BotUser
	if err := r.db.Where("telegram_id = ?", telegramID).First(&user).Error; err != nil {
		return nil, err
	}
	return &user, nil
}

func (r *gormBotUserRepository) FindByID(id uint) (*models.BotUser, error) {
	var user models.BotUser
	if err := r.db.Where("id = ?", id).First(&user).Error; err != nil {
		return nil, err
	}
	return &user, nil
}

func (r *gormBotUserRepository) Create(user *models.BotUser) error {
	return r.db.Create(user).Error
}

func (r *gormBotUserRepository) Update(user *models.BotUser) error {
	return r.db.Save(user).Error
}

func (r *gormBotUserRepository) UpdateCaptchaStatus(user *models.BotUser, hasPassed bool) error {
	user.HasPassedCaptcha = hasPassed
	return r.db.Save(user).Error
}

func (r *gormBotUserRepository) UpdateBalance(userID uint, amount float64) error {
	return r.db.Model(&models.BotUser{}).Where("id = ?", userID).Update("balance", gorm.Expr("balance + ?", amount)).Error
}

func (r *gormBotUserRepository) GetUserBalance(userID uint) (float64, error) {
	var user models.BotUser
	if err := r.db.Model(&models.BotUser{}).Where("id = ?", userID).Select("balance").First(&user).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return 0, nil // Return 0 if user not found, consistent with old logic
		}
		return 0, err
	}
	return user.Balance, nil
}

func (r *gormBotUserRepository) GetUserTransactions(userID uint) ([]models.Transaction, error) {
	var transactions []models.Transaction
	err := r.db.Where("user_id = ?", userID).Order("created_at desc").Find(&transactions).Error
	return transactions, err
}

func (r *gormBotUserRepository) UpdateBotUserStatus(telegramID int64, updates map[string]interface{}) error {
	return r.db.Model(&models.BotUser{}).Where("telegram_id = ?", telegramID).Updates(updates).Error
}
