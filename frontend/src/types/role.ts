export interface Role {
  id: number;
  name: string;
  is_super: boolean;
}

export interface Permission {
  id: number;
  name: string;
  group: string;
}

export interface UserPermission {
  user_id: number;
  permission_id: number;
  effect: 'allow' | 'deny';
}
