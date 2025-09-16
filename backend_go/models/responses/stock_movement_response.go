package responses

import (
	"time"

	"frbktg/backend_go/models"
)

type StockMovementResponse struct {
	ID          uint                   `json:"id"`
	ProductID   uint                   `json:"product_id"`
	Type        models.StockMovementType `json:"type"`
	Quantity    int                    `json:"quantity"`
	CreatedAt   time.Time              `json:"created_at"`
	Description string                 `json:"description"`
	OrderID     *uint                  `json:"order_id"`
}
