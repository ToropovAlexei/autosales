package repositories

import (
	"frbktg/backend_go/models"
	"gorm.io/gorm"
	"time"
)

type UserSubscriptionRepository interface {
	WithTx(tx *gorm.DB) UserSubscriptionRepository
	FindActiveSubscription(botUserID, productID uint) (*models.UserSubscription, error)
	FindActiveSubscriptionByID(id uint) (*models.UserSubscription, error)
	GetExpiringSubscriptions(within time.Duration) ([]models.UserSubscription, error)
	FindSubscriptionsByBotUserID(botUserID uint) ([]models.UserSubscription, error)
	CreateSubscription(subscription *models.UserSubscription) error
	UpdateSubscription(subscription *models.UserSubscription) error
}

type gormUserSubscriptionRepository struct {
	db *gorm.DB
}

func NewUserSubscriptionRepository(db *gorm.DB) UserSubscriptionRepository {
	return &gormUserSubscriptionRepository{db: db}
}

func (r *gormUserSubscriptionRepository) WithTx(tx *gorm.DB) UserSubscriptionRepository {
	return &gormUserSubscriptionRepository{db: tx}
}

func (r *gormUserSubscriptionRepository) FindActiveSubscription(botUserID, productID uint) (*models.UserSubscription, error) {
	var subscription models.UserSubscription
	err := r.db.Where("bot_user_id = ? AND product_id = ? AND is_active = ? AND expires_at > ?", botUserID, productID, true, time.Now()).First(&subscription).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, nil // Not found is not an error here
		}
		return nil, err
	}
	return &subscription, nil
}

func (r *gormUserSubscriptionRepository) FindActiveSubscriptionByID(id uint) (*models.UserSubscription, error) {
	var subscription models.UserSubscription
	err := r.db.Where("id = ? AND is_active = ?", id, true).First(&subscription).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, nil // Not found is not an error here
		}
		return nil, err
	}
	return &subscription, nil
}

func (r *gormUserSubscriptionRepository) GetExpiringSubscriptions(within time.Duration) ([]models.UserSubscription, error) {
	var subscriptions []models.UserSubscription
	now := time.Now()
	expiresAtLimit := now.Add(within)
	err := r.db.Where("is_active = ? AND expires_at BETWEEN ? AND ?", true, now, expiresAtLimit).Find(&subscriptions).Error
	if err != nil {
		return nil, err
	}
	return subscriptions, nil
}

func (r *gormUserSubscriptionRepository) FindSubscriptionsByBotUserID(botUserID uint) ([]models.UserSubscription, error) {
	var subscriptions []models.UserSubscription
	if err := r.db.Preload("Product").Where("bot_user_id = ?", botUserID).Order("created_at desc").Find(&subscriptions).Error; err != nil {
		return nil, err
	}
	return subscriptions, nil
}

func (r *gormUserSubscriptionRepository) CreateSubscription(subscription *models.UserSubscription) error {
	return r.db.Create(subscription).Error
}

func (r *gormUserSubscriptionRepository) UpdateSubscription(subscription *models.UserSubscription) error {
	return r.db.Save(subscription).Error
}
