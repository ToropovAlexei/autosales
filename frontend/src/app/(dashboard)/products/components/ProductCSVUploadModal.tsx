"use client";

import { useState } from "react";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  LinearProgress,
  Alert,
  AlertTitle,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
} from "@mui/material";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { queryKeys } from "@/utils/query";
import { UploadBtn } from "@/components";
import { toast } from "react-toastify";

interface ProductCSVUploadModalProps {
  open: boolean;
  onClose: () => void;
}

export const ProductCSVUploadModal = ({
  open,
  onClose,
}: ProductCSVUploadModalProps) => {
  const queryClient = useQueryClient();
  const [selectedFile, setSelectedFile] = useState<File | null>(null);

  const { data, mutate, isPending } = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData();
      formData.append("file", file);
      return dataLayer.create<{
        created: number | null;
        failed: number;
        skipped: number;
        errors: string[] | null;
      }>({
        url: ENDPOINTS.PRODUCTS_UPLOAD_CSV,
        params: formData,
      });
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.PRODUCTS),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.CATEGORIES),
      });
      setSelectedFile(null);
    },
    onError: () => {
      toast.error("Произошла ошибка при загрузке CSV-файла");
    },
  });

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files && event.target.files[0]) {
      setSelectedFile(event.target.files[0]);
    } else {
      setSelectedFile(null);
    }
  };

  const handleUploadClick = () => {
    if (selectedFile) {
      mutate(selectedFile);
    }
  };

  const handleClose = () => {
    setSelectedFile(null);
    onClose();
  };

  return (
    <Dialog open={open} onClose={handleClose} fullWidth maxWidth="sm">
      <DialogTitle>Загрузить товары из CSV</DialogTitle>
      <DialogContent dividers>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          Пожалуйста, загрузите CSV-файл со следующими столбцами (в указанном
          порядке):
        </Typography>
        <Table size="small">
          <TableHead>
            <TableRow>
              <TableCell>name</TableCell>
              <TableCell>category</TableCell>
              <TableCell>price</TableCell>
              <TableCell>initial_stock</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            <TableRow>
              <TableCell>Google Pixel</TableCell>
              <TableCell>
                Электроника-&gt;Телефоны-&gt;Android-&gt;Google
              </TableCell>
              <TableCell>30000</TableCell>
              <TableCell>10</TableCell>
            </TableRow>
          </TableBody>
        </Table>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          Разделитель категорий вложенности: <code>-&gt;</code> (например,{" "}
          <code>Электроника-&gt;Телефоны-&gt;Android</code>)
        </Typography>
        <UploadBtn
          onFileChange={handleFileChange}
          accept=".csv"
          loading={isPending}
        >
          {selectedFile ? `Выбран ${selectedFile.name}` : "Выбрать CSV файл"}
        </UploadBtn>

        {data && (
          <Alert
            severity={data.failed > 0 ? "error" : "success"}
            sx={{ mt: 2 }}
          >
            <AlertTitle>Результат загрузки</AlertTitle>
            <Typography>Успешно создано: {data.created}</Typography>
            <Typography>Пропущено: {data.skipped}</Typography>
            <Typography>Ошибок: {data.failed}</Typography>
            {(data.errors?.length || 0) > 0 && (
              <ul>
                {data.errors?.map((err, index) => (
                  <li key={index}>{err}</li>
                ))}
              </ul>
            )}
          </Alert>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Отмена</Button>
        <Button
          onClick={handleUploadClick}
          disabled={!selectedFile}
          loading={isPending}
          variant="contained"
        >
          Загрузить
        </Button>
      </DialogActions>
    </Dialog>
  );
};
