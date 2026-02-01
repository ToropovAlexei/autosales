import { ENDPOINTS } from "@/constants";
import { useList } from "@/hooks";
import {
  translatePermission,
  translatePermissionGroup,
} from "@/lib/permissions";
import {
  Checkbox,
  FormControlLabel,
  FormGroup,
  Skeleton,
  Typography,
} from "@mui/material";
import { PERMISSIONS_COLORS } from "./constants";
import classes from "./styles.module.css";
import { range } from "@/utils/array";
import { Fragment } from "react/jsx-runtime";
import { PermissionName, PermissionResponse } from "@/types";

interface IProps {
  value: PermissionResponse["id"][];
  onChange: (permissionId: PermissionResponse["id"], checked: boolean) => void;
  onChangeAll: (checked: boolean) => void;
  loading?: boolean;
}

export const PermissionsSelector = ({
  onChange,
  onChangeAll,
  value,
  loading,
}: IProps) => {
  const { data, isPending } = useList<PermissionResponse>({
    endpoint: ENDPOINTS.PERMISSIONS,
  });

  const groupedPermissions = Object.groupBy(
    data?.data || [],
    (permission) => permission.group || "Other",
  );

  const selectedPermissions = new Set(value);

  if (loading || isPending) {
    return (
      <div className={classes.container}>
        {range(5).map((key) => (
          <Fragment key={key}>
            <Skeleton height={25} variant="rounded" width="40%" />
            <div style={{ display: "grid", gap: "0.25rem" }}>
              {range(3).map((key) => (
                <Skeleton key={key} height={20} variant="rounded" />
              ))}
            </div>
          </Fragment>
        ))}
      </div>
    );
  }

  return (
    <div className={classes.container}>
      <FormControlLabel
        control={
          <Checkbox
            checked={selectedPermissions.size === data?.data?.length}
            onChange={(_, checked) => onChangeAll(checked)}
          />
        }
        label="! Полный доступ"
        slotProps={{ typography: { color: "error" } }}
      />
      {Object.entries(groupedPermissions).map(([group, permissions]) => (
        <div key={group}>
          <Typography variant="h6" fontSize="1rem">
            {translatePermissionGroup(group)}
          </Typography>
          <FormGroup>
            {permissions?.map((permission) => (
              <FormControlLabel
                key={permission.id}
                control={
                  <Checkbox
                    checked={selectedPermissions.has(permission.id)}
                    onChange={(_, checked) => onChange(permission.id, checked)}
                  />
                }
                label={translatePermission(permission.name as PermissionName)}
                slotProps={{
                  typography: {
                    color: PERMISSIONS_COLORS[permission.name] || "success",
                  },
                }}
              />
            ))}
          </FormGroup>
        </div>
      ))}
    </div>
  );
};
