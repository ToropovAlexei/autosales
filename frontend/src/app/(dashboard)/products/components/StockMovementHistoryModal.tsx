"use client";

import {
  Dialog,
  DialogTitle,
  DialogContent,
  IconButton,
  Typography,
  Box,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import dayjs from "dayjs";
import { StockMovement } from "@/types";

interface StockMovementHistoryModalProps {
  open: boolean;
  onClose: () => void;
  productId: bigint | null;
}

export const StockMovementHistoryModal = ({
  open,
  onClose,
  productId,
}: StockMovementHistoryModalProps) => {
  const { data: stockMovementsData, isPending: isLoadingStockMovements } =
    useList<StockMovement>({
      endpoint: ENDPOINTS.STOCK_MOVEMENTS,
      filter: {
        filters: [{ field: "product_id", op: "eq", value: productId }],
        orderBy: "id",
        order: "desc",
      },
      enabled: open && productId !== null,
    });

  const stockMovements = stockMovementsData?.data || [];

  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90 },
    {
      field: "type",
      headerName: "Тип",
      width: 150,
      valueFormatter: (value) => {
        switch (value) {
          case "initial":
            return "Начальный";
          case "sale":
            return "Продажа";
          case "return":
            return "Возврат";
          case "adjustment":
            return "Корректировка";
          default:
            return value;
        }
      },
    },
    {
      field: "quantity",
      headerName: "Количество",
      type: "number",
      width: 120,
      renderCell: (params) => {
        return (
          <Typography
            color={params.value > 0 ? "success" : "error"}
            component="span"
            align="center"
          >
            {params.value > 0 ? `+${params.value}` : params.value}
          </Typography>
        );
      },
    },
    { field: "description", headerName: "Описание", flex: 1 },
    {
      field: "created_at",
      headerName: "Дата",
      width: 180,
      valueFormatter: (value) => dayjs(value).format("DD.MM.YYYY HH:mm"),
    },
  ];

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>
        <Box display="flex" alignItems="center" justifyContent="space-between">
          <Typography variant="h6">История движений по складу</Typography>
          <IconButton onClick={onClose}>
            <CloseIcon />
          </IconButton>
        </Box>
      </DialogTitle>
      <DialogContent>
        {isLoadingStockMovements ? (
          <Typography>Загрузка...</Typography>
        ) : (
          <div style={{ height: 400, width: "100%" }}>
            <DataGrid
              rows={stockMovements}
              columns={columns}
              pageSizeOptions={[5, 10, 20]}
              disableRowSelectionOnClick
              localeText={ruRU.components.MuiDataGrid.defaultProps.localeText}
            />
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
};
