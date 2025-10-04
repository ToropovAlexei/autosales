import { DataGrid, GridColDef } from '@mui/x-data-grid';
import { IProduct } from '@/types';
import classes from './TopProductsTable.module.css';

interface TopProductsTableProps {
  products: IProduct[];
}

export const TopProductsTable = ({ products }: TopProductsTableProps) => {
  const columns: GridColDef[] = [
    { field: 'name', headerName: 'Название', flex: 1 },
    { field: 'price', headerName: 'Цена', width: 120 },
    { field: 'total_revenue', headerName: 'Выручка', width: 150 },
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
