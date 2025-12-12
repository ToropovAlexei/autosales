import { PermissionName } from "./permissions";

export interface Role {
  id: number;
  name: string;
  is_super: boolean;
  permissions: Permission[];
}

export interface Permission {
  id: number;
  name: PermissionName;
  group: string;
}

export interface UserPermission {
  user_id: number;
  permission_id: number;
  effect: "allow" | "deny";
}
