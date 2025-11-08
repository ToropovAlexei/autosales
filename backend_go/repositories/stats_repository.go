package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type StatsRepository interface {
	WithTx(tx *gorm.DB) StatsRepository
	GetReferralStats(ownerID uint) (map[uint]models.ReferralBotStats, error)
	GetReferralTurnover(ownerID uint) (float64, error)
	GetReferralAccruals(ownerID uint) (float64, error)
}

type gormStatsRepository struct {
	db *gorm.DB
}

func NewStatsRepository(db *gorm.DB) StatsRepository {
	return &gormStatsRepository{db: db}
}

func (r *gormStatsRepository) WithTx(tx *gorm.DB) StatsRepository {
	return &gormStatsRepository{db: r.db.Begin()}
}

func (r *gormStatsRepository) GetReferralStats(ownerID uint) (map[uint]models.ReferralBotStats, error) {
	var stats []models.ReferralBotStats
	if err := r.db.Table("bots").
		Select("bots.id as bot_id, COALESCE(SUM(ref_transactions.ref_share), 0) as total_earnings, COUNT(DISTINCT orders.id) as purchase_count").
		Joins("left join orders on orders.bot_id = bots.id").
		Joins("left join ref_transactions on ref_transactions.order_id = orders.id").
		Where("bots.owner_id = ?", ownerID).
		Group("bots.id").
		Scan(&stats).Error; err != nil {
		return nil, err
	}

	statsMap := make(map[uint]models.ReferralBotStats)
	for _, s := range stats {
		statsMap[s.BotID] = s
	}

	return statsMap, nil
}

func (r *gormStatsRepository) GetReferralTurnover(ownerID uint) (float64, error) {
	var turnover float64
	err := r.db.Table("orders").
		Joins("join bots on orders.bot_id = bots.id").
		Where("bots.owner_id = ?", ownerID).
		Select("COALESCE(SUM(orders.amount), 0)").
		Row().
		Scan(&turnover)
	return turnover, err
}

func (r *gormStatsRepository) GetReferralAccruals(ownerID uint) (float64, error) {
	var accruals float64
	err := r.db.Table("ref_transactions").
		Where("ref_owner_id = ?", ownerID).
		Select("COALESCE(SUM(ref_transactions.ref_share), 0)").
		Row().
		Scan(&accruals)
	return accruals, err
}
