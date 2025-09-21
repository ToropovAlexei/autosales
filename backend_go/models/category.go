package models

// Category представляет собой иерархическую структуру категорий

type Category struct {
	ID            uint       `gorm:"primaryKey"`
	Name          string     `gorm:"index"`
	ParentID      *uint      `gorm:"index"` // Указатель, чтобы разрешить NULL
	Parent        *Category  `gorm:"foreignKey:ParentID"`
	SubCategories []Category `gorm:"foreignKey:ParentID"`
	Products      []Product  `gorm:"foreignKey:CategoryID"`
}

// CategoryResponse определяет, как категория и ее вложенные подкатегории
// будут представлены в JSON-ответе API.
type CategoryResponse struct {
	ID            uint               `json:"id"`
	Name          string             `json:"name"`
	ParentID      *uint              `json:"parent_id,omitempty"`
	SubCategories []CategoryResponse `json:"sub_categories,omitempty"`
}
