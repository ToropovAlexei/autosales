package repositories

import (
	"database/sql"
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
	GetUserBalance(userID uint) (float64, error)
	GetUserTransactions(userID uint) ([]models.Transaction, error)
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
	if err := r.db.Where("id = ? AND is_deleted = ?", id, false).First(&user).Error; err != nil {
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

func (r *gormBotUserRepository) GetUserBalance(userID uint) (float64, error) {
	var balance sql.NullFloat64
	if err := r.db.Model(&models.Transaction{}).Where("user_id = ?", userID).Select("sum(amount)").
		Row().Scan(&balance); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return 0, err
	}

	if balance.Valid {
		return balance.Float64, nil
	}

	return 0.0, nil
}

func (r *gormBotUserRepository) GetUserTransactions(userID uint) ([]models.Transaction, error) {
	var transactions []models.Transaction
	err := r.db.Where("user_id = ?", userID).Order("created_at desc").Find(&transactions).Error
	return transactions, err
}
