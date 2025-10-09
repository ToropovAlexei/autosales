package models

// Setting представляет собой пару ключ-значение для настроек магазина.
type Setting struct {
	Key   string `gorm:"primaryKey"` // Ключ настройки (например, "store_name")
	Value string `gorm:"type:text"`  // Значение настройки
}
