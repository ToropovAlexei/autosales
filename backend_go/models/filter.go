package models

// Filter представляет один критерий фильтрации.
type Filter struct {
	Field    string `json:"field"`
	Operator string `json:"op"`
	Value    any    `json:"value"`
}
