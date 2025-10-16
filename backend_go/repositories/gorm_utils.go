package repositories

import (
	"fmt"
	"frbktg/backend_go/models"
	"strings"

	"gorm.io/gorm"
)

func ApplyPagination[T any](db *gorm.DB, page models.Page) (*models.PaginatedResult[T], error) {
	var total int64
	if err := db.Model(new(T)).Count(&total).Error; err != nil {
		return nil, err
	}

	offset := (page.Page - 1) * page.PageSize
	order := fmt.Sprintf("%s %s", page.OrderBy, page.Order)

	var data []T
	if err := db.Order(order).Limit(page.PageSize).Offset(offset).Find(&data).Error; err != nil {
		return nil, err
	}

	return &models.PaginatedResult[T]{
		Data:  data,
		Total: total,
	}, nil
}

func ApplyFilters[T any](db *gorm.DB, filters []models.Filter) *gorm.DB {
	stmt := &gorm.Statement{DB: db}
	if err := stmt.Parse(new(T)); err != nil {
		// If schema parsing fails, we can't validate fields.
		// Returning the original query is a safe fallback.
		return db
	}

	for _, filter := range filters {
		// Check if the field exists as a database column in the model's schema.
		if _, ok := stmt.Schema.FieldsByDBName[filter.Field]; !ok {
			continue // Ignore unsupported filter fields.
		}

		op := strings.ToLower(filter.Operator)
		switch op {
		case "=", "<", ">", "<=", ">=", "!=":
			db = db.Where(fmt.Sprintf("%s %s ?", filter.Field, op), filter.Value)
		case "like", "contains":
			db = db.Where(fmt.Sprintf("%s LIKE ?", filter.Field), fmt.Sprintf("%%%v%%", filter.Value))
		case "in":
			db = db.Where(fmt.Sprintf("%s IN (?)", filter.Field), filter.Value)
		}
	}
	return db
}
