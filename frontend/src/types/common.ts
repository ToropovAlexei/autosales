import { Role } from "@/types/role";

export interface IFilter {
  page?: number;
  pageSize?: number;
  filters?: {
    field: string;
    op: string;
    value: any;
  }[];
  start_date?: string;
  end_date?: string;
}

export type FalsyValues = false | null | 0 | "" | undefined;

export type User = {
  id: number;
  login: string;
  is_active: boolean;
  roles: Role[];
};

export interface BotUser {
  id: number;
  telegram_id: number;
  balance: number;
  is_blocked: boolean;
  bot_is_blocked_by_user: boolean;
  registered_with_bot: string;
  last_seen_with_bot: string;
  created_at: string;
  last_seen_at: string;
}
