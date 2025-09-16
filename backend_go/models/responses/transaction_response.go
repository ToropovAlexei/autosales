package responses

import (
	"time"

	"frbktg/backend_go/models"
)

type TransactionResponse struct {
	ID          uint                   `json:"id"`
	UserID      uint                   `json:"user_id"`
	OrderID     *uint                  `json:"order_id"`
	Type        models.TransactionType `json:"type"`
	Amount      float64                `json:"amount"`
	CreatedAt   time.Time              `json:"created_at"`
	Description string                 `json:"description"`
}
