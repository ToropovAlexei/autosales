import { DataGrid, GridColDef } from "@mui/x-data-grid";
import classes from "./TopProductsTable.module.css";
import { Product } from "@/types";

interface TopProductsTableProps {
  products: Product[];
}

export const TopProductsTable = ({ products }: TopProductsTableProps) => {
  const columns: GridColDef[] = [
    { field: "name", headerName: "Название", flex: 1 },
    { field: "price", headerName: "Цена", width: 120 },
    { field: "total_revenue", headerName: "Выручка", width: 150 },
  ];

  return (
    <div className={classes.tableContainer}>
      <DataGrid
        rows={products}
        columns={columns}
        density="compact"
        hideFooter
      />
    </div>
  );
};
