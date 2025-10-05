export interface IFilter {
  [key: string]:
    | string
    | number
    | string[]
    | number[]
    | boolean
    | undefined
    | null;
}

export type FalsyValues = false | null | 0 | "" | undefined;

export interface User {
  id: number;
  email: string;
  is_active: boolean;
  role: string;
  referral_program_enabled: boolean;
  referral_percentage: number;
}

export interface BotUser {
  id: number;
  telegram_id: number;
  balance: number;
  registered_with_bot: string;
  last_seen_with_bot: string;
  created_at: string;
}
