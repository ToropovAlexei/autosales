package repositories

import (
	"errors"
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type ReferralRepository interface {
	WithTx(tx *gorm.DB) ReferralRepository
	CountByOwnerID(ownerID uint) (int64, error)
	GetBotsByOwnerID(ownerID uint) ([]models.ReferralBot, error)
	SetPrimary(ownerID, botID uint) error
	UpdatePercentageForOwner(ownerID uint, percentage float64) error
	FindByBotToken(botToken string) (*models.ReferralBot, error)
	CreateReferralBot(bot *models.ReferralBot) error
	GetAllReferralBots() ([]models.ReferralBot, error)
	GetAdminInfoForOwner(ownerID uint) ([]models.ReferralBotAdminInfo, error)
	GetAllAdminInfo() ([]models.ReferralBotAdminInfo, error)
	GetReferralBotByID(id uint) (*models.ReferralBot, error)
	UpdateReferralBot(bot *models.ReferralBot) error
	DeleteReferralBot(bot *models.ReferralBot) error
}

type gormReferralRepository struct {
	db *gorm.DB
}

func NewReferralRepository(db *gorm.DB) ReferralRepository {
	return &gormReferralRepository{db: db}
}

func (r *gormReferralRepository) UpdatePercentageForOwner(ownerID uint, percentage float64) error {
	return r.db.Model(&models.ReferralBot{}).Where("owner_id = ?", ownerID).Update("referral_percentage", percentage).Error
}


func (r *gormReferralRepository) WithTx(tx *gorm.DB) ReferralRepository {
	return &gormReferralRepository{db: tx}
}

func (r *gormReferralRepository) CountByOwnerID(ownerID uint) (int64, error) {
	var count int64
	err := r.db.Model(&models.ReferralBot{}).Where("owner_id = ?", ownerID).Count(&count).Error
	return count, err
}

func (r *gormReferralRepository) GetBotsByOwnerID(ownerID uint) ([]models.ReferralBot, error) {
	var bots []models.ReferralBot
	if err := r.db.Where("owner_id = ?", ownerID).Find(&bots).Error; err != nil {
		return nil, err
	}
	return bots, nil
}

func (r *gormReferralRepository) FindByBotToken(botToken string) (*models.ReferralBot, error) {
	var bot models.ReferralBot
	err := r.db.Where("bot_token = ?", botToken).First(&bot).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &bot, nil
}

func (r *gormReferralRepository) CreateReferralBot(bot *models.ReferralBot) error {
	return r.db.Create(bot).Error
}

func (r *gormReferralRepository) GetAllReferralBots() ([]models.ReferralBot, error) {
	var bots []models.ReferralBot
	if err := r.db.Order("id asc").Find(&bots).Error; err != nil {
		return nil, err
	}
	return bots, nil
}

func (r *gormReferralRepository) GetAdminInfoForOwner(ownerID uint) ([]models.ReferralBotAdminInfo, error) {
	var bots []models.ReferralBotAdminInfo

	err := r.db.Table("referral_bots").
		Select(
			"referral_bots.id, referral_bots.owner_id, referral_bots.referral_percentage, "+
				"referral_bots.bot_token, referral_bots.is_active, referral_bots.is_primary, referral_bots.created_at, "+
				"bot_users.telegram_id as owner_telegram_id, "+
				"COALESCE(SUM(ref_transactions.amount), 0) as turnover, "+
				"COALESCE(SUM(ref_transactions.ref_share), 0) as accruals",
		).
		Joins("join bot_users on referral_bots.owner_id = bot_users.id").
		Joins("left join ref_transactions on referral_bots.owner_id = ref_transactions.ref_owner_id").
		Where("referral_bots.owner_id = ?", ownerID).
		Group("referral_bots.id, bot_users.telegram_id").
		Order("referral_bots.id asc").
		Scan(&bots).Error

	if err != nil {
		return nil, err
	}
	if bots == nil {
		return []models.ReferralBotAdminInfo{}, nil
	}

	return bots, err
}

func (r *gormReferralRepository) GetAllAdminInfo() ([]models.ReferralBotAdminInfo, error) {
	var bots []models.ReferralBotAdminInfo

	err := r.db.Table("referral_bots").
		Select(
			"referral_bots.id, referral_bots.owner_id, referral_bots.referral_percentage, "+
				"referral_bots.bot_token, referral_bots.is_active, referral_bots.is_primary, referral_bots.created_at, "+
				"bot_users.telegram_id as owner_telegram_id, "+
				"COALESCE(SUM(ref_transactions.amount), 0) as turnover, "+
				"COALESCE(SUM(ref_transactions.ref_share), 0) as accruals",
		).
		Joins("join bot_users on referral_bots.owner_id = bot_users.id").
		Joins("left join ref_transactions on referral_bots.owner_id = ref_transactions.ref_owner_id").
		Group("referral_bots.id, bot_users.telegram_id").
		Order("referral_bots.id asc").
		Scan(&bots).Error

	if err != nil {
		return nil, err
	}
	if bots == nil {
		return []models.ReferralBotAdminInfo{}, nil
	}

	return bots, err
}

func (r *gormReferralRepository) GetReferralBotByID(id uint) (*models.ReferralBot, error) {
	var bot models.ReferralBot
	if err := r.db.First(&bot, id).Error; err != nil {
		return nil, err
	}
	return &bot, nil
}

func (r *gormReferralRepository) UpdateReferralBot(bot *models.ReferralBot) error {
	return r.db.Save(bot).Error
}

func (r *gormReferralRepository) SetPrimary(ownerID, botID uint) error {
	return r.db.Transaction(func(tx *gorm.DB) error {
		// Шаг 1: Сбросить флаг IsPrimary для всех ботов этого продавца
		if err := tx.Model(&models.ReferralBot{}).Where("owner_id = ?", ownerID).Update("is_primary", false).Error; err != nil {
			return err
		}

		// Шаг 2: Установить флаг IsPrimary для выбранного бота
		if err := tx.Model(&models.ReferralBot{}).Where("id = ? AND owner_id = ?", botID, ownerID).Update("is_primary", true).Error; err != nil {
			return err
		}

		return nil
	})
}

func (r *gormReferralRepository) DeleteReferralBot(bot *models.ReferralBot) error {
	return r.db.Delete(bot).Error
}
