export interface BroadcastForm {
  text: string;
  image_id: string | null;
  balance_min?: number | null;
  balance_max?: number | null;
  registered_after?: string | null;
  registered_before?: string | null;
  last_seen_after?: string | null;
  last_seen_before?: string | null;
  bot_name?: string | null;
}
