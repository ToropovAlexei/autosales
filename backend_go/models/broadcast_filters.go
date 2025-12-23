package models

import "time"

// BroadcastFilters defines the query parameters for filtering users for a broadcast.
type BroadcastFilters struct {
	BalanceMin       *float64   `form:"balance_min" json:"balance_min,omitempty"`
	BalanceMax       *float64   `form:"balance_max" json:"balance_max,omitempty"`
	RegisteredAfter  *time.Time `form:"registered_after" time_format:"2006-01-02T15:04:05Z07:00" json:"registered_after,omitempty"`
	RegisteredBefore *time.Time `form:"registered_before" time_format:"2006-01-02T15:04:05Z07:00" json:"registered_before,omitempty"`
	LastSeenAfter    *time.Time `form:"last_seen_after" time_format:"2006-01-02T15:04:05Z07:00" json:"last_seen_after,omitempty"`
	LastSeenBefore   *time.Time `form:"last_seen_before" time_format:"2006-01-02T15:04:05Z07:00" json:"last_seen_before,omitempty"`
	BotName          *string    `form:"bot_name" json:"bot_name,omitempty"`
}
