package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type ReferralRepository interface {
	WithTx(tx *gorm.DB) ReferralRepository
	FindByBotToken(botToken string) (*models.ReferralBot, error)
	CreateReferralBot(bot *models.ReferralBot) error
	FindReferralBotByToken(botToken string) (*models.ReferralBot, error)
	GetAllReferralBots() ([]models.ReferralBot, error)
	GetAdminInfoForSeller(sellerID uint) ([]models.ReferralBotAdminInfo, error)
	GetReferralBotByID(id uint) (*models.ReferralBot, error)
	UpdateReferralBot(bot *models.ReferralBot) error
}

type gormReferralRepository struct {
	db *gorm.DB
}

func NewReferralRepository(db *gorm.DB) ReferralRepository {
	return &gormReferralRepository{db: db}
}

func (r *gormReferralRepository) WithTx(tx *gorm.DB) ReferralRepository {
	return &gormReferralRepository{db: tx}
}

func (r *gormReferralRepository) FindByBotToken(botToken string) (*models.ReferralBot, error) {
	var bot models.ReferralBot
	if err := r.db.Where("bot_token = ?", botToken).First(&bot).Error; err != nil {
		return nil, err
	}
	return &bot, nil
}

func (r *gormReferralRepository) CreateReferralBot(bot *models.ReferralBot) error {
	return r.db.Create(bot).Error
}

func (r *gormReferralRepository) FindReferralBotByToken(botToken string) (*models.ReferralBot, error) {
	var bot models.ReferralBot
	if err := r.db.Where("bot_token = ?", botToken).First(&bot).Error; err != nil {
		return nil, err
	}
	return &bot, nil
}

func (r *gormReferralRepository) GetAllReferralBots() ([]models.ReferralBot, error) {
	var bots []models.ReferralBot
	if err := r.db.Find(&bots).Error; err != nil {
		return nil, err
	}
	return bots, nil
}

func (r *gormReferralRepository) GetAdminInfoForSeller(sellerID uint) ([]models.ReferralBotAdminInfo, error) {
	var bots []models.ReferralBotAdminInfo

	err := r.db.Table("referral_bots").
		Select(
			"referral_bots.id, referral_bots.owner_id, referral_bots.seller_id, "+
				"referral_bots.bot_token, referral_bots.is_active, referral_bots.created_at, "+
				"bot_users.telegram_id as owner_telegram_id, "+
				"COALESCE(SUM(ref_transactions.amount), 0) as turnover, "+
				"COALESCE(SUM(ref_transactions.ref_share), 0) as accruals",
		).
		Joins("join bot_users on referral_bots.owner_id = bot_users.id").
		Joins("left join ref_transactions on referral_bots.owner_id = ref_transactions.ref_owner_id").
		Where("referral_bots.seller_id = ?", sellerID).
		Group("referral_bots.id, bot_users.telegram_id").
		Scan(&bots).Error

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
