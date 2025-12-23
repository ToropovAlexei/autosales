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
	GetBotUsersForBroadcast(filters models.BroadcastFilters, page models.Page) (*models.PaginatedResult[models.BotUser], error)
	GetAllBotUsersForBroadcast(filters models.BroadcastFilters) ([]models.BotUser, error)
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

func (r *gormBotUserRepository) GetBotUsersForBroadcast(filters models.BroadcastFilters, page models.Page) (*models.PaginatedResult[models.BotUser], error) {
	var users []models.BotUser
	var total int64

	query := r.db.Model(&models.BotUser{})

	// Always exclude blocked users
	query = query.Where("is_blocked = ? AND bot_is_blocked_by_user = ?", false, false)

	// Apply filters
	if filters.BalanceMin != nil {
		query = query.Where("balance >= ?", *filters.BalanceMin)
	}
	if filters.BalanceMax != nil {
		query = query.Where("balance <= ?", *filters.BalanceMax)
	}
	if filters.RegisteredAfter != nil {
		query = query.Where("created_at >= ?", *filters.RegisteredAfter)
	}
	if filters.RegisteredBefore != nil {
		query = query.Where("created_at <= ?", *filters.RegisteredBefore)
	}
	if filters.LastSeenAfter != nil {
		query = query.Where("last_seen_at >= ?", *filters.LastSeenAfter)
	}
	if filters.LastSeenBefore != nil {
		query = query.Where("last_seen_at <= ?", *filters.LastSeenBefore)
	}
	if filters.BotName != nil && *filters.BotName != "" {
		// Assuming we filter by the bot they registered with
		query = query.Where("registered_with_bot = ?", *filters.BotName)
	}

	// Count total records that match the filters
	if err := query.Count(&total).Error; err != nil {
		return nil, err
	}

	// Apply pagination
	offset := (page.Page - 1) * page.PageSize
	if err := query.Limit(page.PageSize).Offset(offset).Find(&users).Error; err != nil {
		return nil, err
	}

	return &models.PaginatedResult[models.BotUser]{
		Data:  users,
		Total: total,
	}, nil
}

func (r *gormBotUserRepository) GetAllBotUsersForBroadcast(filters models.BroadcastFilters) ([]models.BotUser, error) {
	var users []models.BotUser

	query := r.db.Model(&models.BotUser{})

	// Always exclude blocked users
	query = query.Where("is_blocked = ? AND bot_is_blocked_by_user = ?", false, false)

	// Apply filters
	if filters.BalanceMin != nil {
		query = query.Where("balance >= ?", *filters.BalanceMin)
	}
	if filters.BalanceMax != nil {
		query = query.Where("balance <= ?", *filters.BalanceMax)
	}
	if filters.RegisteredAfter != nil {
		query = query.Where("created_at >= ?", *filters.RegisteredAfter)
	}
	if filters.RegisteredBefore != nil {
		query = query.Where("created_at <= ?", *filters.RegisteredBefore)
	}
	if filters.LastSeenAfter != nil {
		query = query.Where("last_seen_at >= ?", *filters.LastSeenAfter)
	}
	if filters.LastSeenBefore != nil {
		query = query.Where("last_seen_at <= ?", *filters.LastSeenBefore)
	}
	if filters.BotName != nil && *filters.BotName != "" {
		// Assuming we filter by the bot they registered with
		query = query.Where("registered_with_bot = ?", *filters.BotName)
	}

	if err := query.Find(&users).Error; err != nil {
		return nil, err
	}

	return users, nil
}


