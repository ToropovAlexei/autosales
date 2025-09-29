package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type PaymentInvoiceRepository interface {
	WithTx(tx *gorm.DB) PaymentInvoiceRepository
	Create(invoice *models.PaymentInvoice) error
	FindByOrderID(orderID string) (*models.PaymentInvoice, error)
	Update(invoice *models.PaymentInvoice) error
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
