package repositories

import (
	"frbktg/backend_go/models"
	"time"

	"gorm.io/gorm"
)

type PaymentInvoiceRepository interface {
	WithTx(tx *gorm.DB) PaymentInvoiceRepository
	Create(invoice *models.PaymentInvoice) error
	FindByOrderID(orderID string) (*models.PaymentInvoice, error)
	Update(invoice *models.PaymentInvoice) error
	GetPendingInvoicesOlderThan(minutes int) ([]models.PaymentInvoice, error)
	GetPendingInvoices() ([]models.PaymentInvoice, error)
}

type gormPaymentInvoiceRepository struct {
	db *gorm.DB
}

func NewPaymentInvoiceRepository(db *gorm.DB) PaymentInvoiceRepository {
	return &gormPaymentInvoiceRepository{db: db}
}

func (r *gormPaymentInvoiceRepository) WithTx(tx *gorm.DB) PaymentInvoiceRepository {
	return &gormPaymentInvoiceRepository{db: tx}
}

func (r *gormPaymentInvoiceRepository) Create(invoice *models.PaymentInvoice) error {
	return r.db.Create(invoice).Error
}

func (r *gormPaymentInvoiceRepository) FindByOrderID(orderID string) (*models.PaymentInvoice, error) {
	var invoice models.PaymentInvoice
	if err := r.db.Where("order_id = ?", orderID).First(&invoice).Error; err != nil {
		return nil, err
	}
	return &invoice, nil
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
		Where("status = ?", models.InvoiceStatusPending).
		Find(&invoices).Error
	return invoices, err
}
