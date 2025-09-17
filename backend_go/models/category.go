package models

type Category struct {
	ID       uint      `gorm:"primaryKey"`
	Name     string    `gorm:"index"`
	Products []Product `gorm:"foreignKey:CategoryID"`
}

type CategoryResponse struct {
	ID   uint   `json:"id"`
	Name string `json:"name"`
}
