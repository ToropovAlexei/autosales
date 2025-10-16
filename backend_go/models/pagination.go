package models

// Page представляет параметры пагинации.
type Page struct {
	Page     int    `form:"page,default=1"`
	PageSize int    `form:"pageSize,default=10"`
	OrderBy  string `form:"orderBy,default=id"`
	Order    string `form:"order,default=asc"`
}

// PaginatedResult представляет собой пагинированный список элементов.
type PaginatedResult[T any] struct {
	Data  []T   `json:"data"`
	Total int64 `json:"total"`
}
