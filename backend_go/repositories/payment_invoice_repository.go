package repositories

import (
	"fmt"
	"frbktg/backend_go/config"
	"frbktg/backend_go/models"
	"time"

	"gorm.io/gorm"
)

type PaymentInvoiceRepository interface {
	WithTx(tx *gorm.DB) PaymentInvoiceRepository
	Create(invoice *models.PaymentInvoice) error
	FindByOrderID(orderID string) (*models.PaymentInvoice, error)
	FindInvoicesByTelegramID(telegramID int64, page models.Page) (*models.PaginatedResult[models.PaymentInvoice], error)
	FindByID(id uint) (*models.PaymentInvoice, error)
	Update(invoice *models.PaymentInvoice) error
	GetPendingInvoicesOlderThan(minutes int) ([]models.PaymentInvoice, error)
	GetPendingInvoices() ([]models.PaymentInvoice, error)
	FindUnfinished() ([]models.PaymentInvoice, error)
	FindPendingPollable() ([]models.PaymentInvoice, error)
}

type gormPaymentInvoiceRepository struct {
	db     *gorm.DB
	config *config.Config
}

func NewPaymentInvoiceRepository(db *gorm.DB, config *config.Config) PaymentInvoiceRepository {
	return &gormPaymentInvoiceRepository{db: db, config: config}
}

func (r *gormPaymentInvoiceRepository) WithTx(tx *gorm.DB) PaymentInvoiceRepository {
	return &gormPaymentInvoiceRepository{db: tx, config: r.config}
}

func (r *gormPaymentInvoiceRepository) Create(invoice *models.PaymentInvoice) error {
	return r.db.Create(invoice).Error
}

func (r *gormPaymentInvoiceRepository) FindByOrderID(orderID string) (*models.PaymentInvoice, error) {
	var invoice models.PaymentInvoice
	if err := r.db.Preload("BotUser").Where("order_id = ?", orderID).First(&invoice).Error; err != nil {
		return nil, err
	}
	return &invoice, nil
}

func (r *gormPaymentInvoiceRepository) FindByID(id uint) (*models.PaymentInvoice, error) {
	var invoice models.PaymentInvoice
	if err := r.db.Preload("BotUser").First(&invoice, id).Error; err != nil {
		return nil, err
	}
	return &invoice, nil
}

func (r *gormPaymentInvoiceRepository) FindInvoicesByTelegramID(telegramID int64, page models.Page) (*models.PaginatedResult[models.PaymentInvoice], error) {
	var invoices []models.PaymentInvoice
	var total int64

	query := r.db.Model(&models.PaymentInvoice{}).
		Joins("JOIN bot_users ON bot_users.id = payment_invoices.bot_user_id").
		Where("bot_users.telegram_id = ?", telegramID)

	// Count total records
	if err := query.Count(&total).Error; err != nil {
		return nil, err
	}

	// Apply pagination
	offset := (page.Page - 1) * page.PageSize
	order := fmt.Sprintf("%s %s", "payment_invoices.created_at", "desc") // Overriding default order

	if err := query.Order(order).Limit(page.PageSize).Offset(offset).Find(&invoices).Error; err != nil {
		return nil, err
	}

	return &models.PaginatedResult[models.PaymentInvoice]{
		Data:  invoices,
		Total: total,
	}, nil
}

func (r *gormPaymentInvoiceRepository) Update(invoice *models.PaymentInvoice) error {
	return r.db.Save(invoice).Error
}

func (r *gormPaymentInvoiceRepository) GetPendingInvoicesOlderThan(minutes int) ([]models.PaymentInvoice, error) {
	var invoices []models.PaymentInvoice
	err := r.db.
		Preload("BotUser").
		Where("status = ?", models.InvoiceStatusPending).
		Where("was_notification_sent = ?", false).
		Where("created_at < ?", time.Now().Add(-time.Duration(minutes)*time.Minute)).
		Find(&invoices).Error
	return invoices, err
}

func (r *gormPaymentInvoiceRepository) GetPendingInvoices() ([]models.PaymentInvoice, error) {
	var invoices []models.PaymentInvoice
	err := r.db.
		Preload("BotUser").
		Where("status = ?", models.InvoiceStatusPending).
		Find(&invoices).Error
	return invoices, err
}

func (r *gormPaymentInvoiceRepository) FindUnfinished() ([]models.PaymentInvoice, error) {
	return r.GetPendingInvoicesOlderThan(r.config.PaymentNotificationMinutes)
}

func (r *gormPaymentInvoiceRepository) FindPendingPollable() ([]models.PaymentInvoice, error) {
	return r.GetPendingInvoices()
}
