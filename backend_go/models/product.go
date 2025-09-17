package models

type Product struct {
	ID         uint   `gorm:"primaryKey"`
	Name       string `gorm:"index"`
	Price      float64
	CategoryID uint
	Category   Category `gorm:"foreignKey:CategoryID"`
}

type ProductResponse struct {
	ID         uint    `json:"id"`
	Name       string  `json:"name"`
	Price      float64 `json:"price"`
	CategoryID uint    `json:"category_id"`
	Stock      int     `json:"stock"`
}
