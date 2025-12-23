export interface BroadcastFilters {
  balance_min?: number;
  balance_max?: number;
  registered_after?: string;
  registered_before?: string;
  last_seen_after?: string;
  last_seen_before?: string;
  bot_name?: string;
}
