import { ENDPOINTS } from "@/constants";
import { useList } from "@/hooks";
import {
  translatePermission,
  translatePermissionGroup,
} from "@/lib/permissions";
import { Permission } from "@/types";
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

interface IProps {
  value: number[];
  onChange: (permissionId: number, checked: boolean) => void;
  loading?: boolean;
}

export const PermissionsSelector = ({ onChange, value, loading }: IProps) => {
  const { data, isPending } = useList<Permission>({
    endpoint: ENDPOINTS.PERMISSIONS,
  });

  const groupedPermissions = Object.groupBy(
    data?.data || [],
    (permission) => permission.group || "Other"
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
                label={translatePermission(permission.name)}
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
