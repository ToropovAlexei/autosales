import { DataGrid } from "@mui/x-data-grid";
import { COLUMNS } from "./constants";
import { useList } from "@/hooks";
import { BotUser } from "@/types";
import { ENDPOINTS } from "@/constants";
import { ruRU } from "@mui/x-data-grid/locales";
import { useWatch } from "react-hook-form";
import { BroadcastForm } from "../../types";
import { cleanFilters } from "../../utils";
import { useDebouncedValue } from "@tanstack/react-pacer";

export const UsersTable = () => {
  const {
    balance_max,
    balance_min,
    bot_name,
    last_seen_after,
    last_seen_before,
    registered_after,
    registered_before,
  } = useWatch<BroadcastForm>();
  const [debounced] = useDebouncedValue(
    cleanFilters({
      balance_max,
      balance_min,
      bot_name,
      last_seen_after,
      last_seen_before,
      registered_after,
      registered_before,
    }),
    { wait: 500 }
  );

  const { data, isFetching } = useList<BotUser>({
    endpoint: ENDPOINTS.BROADCAST_USERS,
    filter: debounced,
  });

  return (
    <DataGrid
      rows={data?.data || []}
      columns={COLUMNS}
      density="compact"
      loading={isFetching}
      rowCount={data?.total || 0}
      sortingMode="server"
      paginationMode="server"
      filterMode="server"
      localeText={ruRU.components.MuiDataGrid.defaultProps.localeText}
    />
  );
};
