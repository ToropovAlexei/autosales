import { DataGrid } from "@mui/x-data-grid";
import { COLUMNS } from "./constants";
import { useList } from "@/hooks";
import { Bot, Customer } from "@/types";
import { ENDPOINTS } from "@/constants";
import { ruRU } from "@mui/x-data-grid/locales";
import { useWatch } from "react-hook-form";
import { BroadcastForm } from "../../types";
import { formToFilters } from "../../utils";
import { useDebouncedValue } from "@tanstack/react-pacer";
import { keyBy } from "@/utils";

export const UsersTable = () => {
  const form = useWatch<BroadcastForm>();
  const [debounced] = useDebouncedValue(form, { wait: 500 });

  const { data, isFetching } = useList<Customer>({
    endpoint: ENDPOINTS.CUSTOMERS,
    filter: { filters: formToFilters(debounced) },
  });

  const { data: bots } = useList<Bot>({
    endpoint: ENDPOINTS.BOTS,
  });
  const botById = keyBy(bots?.data || [], "id");
  const usersWithBot = (data?.data || []).map((customer) => ({
    ...customer,
    bot_name: botById[Number(customer.registered_with_bot)]?.username,
  }));

  return (
    <DataGrid
      rows={usersWithBot || []}
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
