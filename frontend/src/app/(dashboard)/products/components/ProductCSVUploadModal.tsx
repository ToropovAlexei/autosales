"use client";

import { useState, useMemo } from "react";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  Alert,
  AlertTitle,
  Box,
} from "@mui/material";
import DownloadIcon from "@mui/icons-material/Download";
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
      toast.success(`Загружено: ${data.created} товаров`);
    },
    onError: () => {
      toast.error("Не удалось загрузить CSV-файл");
    },
  });

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files?.[0]) {
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

  // 📄 Генерация CSV-шаблона на лету
  const templateCSV = useMemo(() => {
    const header = ["name", "category", "price", "initial_stock"].join(",");
    const example = [
      "Google Pixel 8", // name
      "Электроника/Телефоны/Android", // category
      "59999.99", // price
      "10", // initial_stock
    ].join(",");
    return `${header}\n${example}\n`;
  }, []);

  const downloadTemplate = () => {
    const blob = new Blob([templateCSV], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.setAttribute("download", "шаблон_товаров.csv");
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  return (
    <Dialog open={open} onClose={handleClose} fullWidth maxWidth="sm">
      <DialogTitle>Добавить товары из CSV</DialogTitle>
      <DialogContent dividers>
        <Typography variant="body2" color="text.secondary" gutterBottom>
          Загрузите CSV-файл с товарами. Не уверены, как его сделать? — скачайте
          шаблон 👇
        </Typography>

        <Box sx={{ display: "flex", gap: 1, my: 2 }}>
          <Button
            variant="outlined"
            startIcon={<DownloadIcon />}
            onClick={downloadTemplate}
            size="small"
          >
            Скачать шаблон CSV
          </Button>
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{ alignSelf: "center" }}
          >
            Откройте в Excel / Google Таблицах → заполните → сохраните как CSV
          </Typography>
        </Box>

        <Typography variant="subtitle2" gutterBottom>
          📋 Требования к файлу:
        </Typography>
        <Typography variant="body2" color="text.secondary" component="div">
          <ul>
            <li>
              Первая строка — заголовки:{" "}
              <code>name,category,price,initial_stock</code>
            </li>
            <li>
              <strong>category</strong> — путь через <code>/</code> (например:{" "}
              <code>Телефоны/Android</code>)
            </li>
            <li>
              <strong>price</strong> — число с точкой: <code>199.99</code>
            </li>
            <li>
              <strong>initial_stock</strong> — целое число ≥ 0
            </li>
          </ul>
        </Typography>

        <UploadBtn
          onFileChange={handleFileChange}
          accept=".csv"
          loading={isPending}
        >
          {selectedFile ? `Выбрано: ${selectedFile.name}` : "Выбрать CSV файл"}
        </UploadBtn>

        {data && (
          <Alert
            severity={data.failed > 0 ? "error" : "success"}
            sx={{ mt: 2 }}
          >
            <AlertTitle>
              {data.failed > 0 ? "Есть ошибки" : "Готово!"}
            </AlertTitle>
            <Typography variant="body2">
              ✅ Успешно: {data.created} &nbsp; ⚠️ Пропущено: {data.skipped}{" "}
              &nbsp; ❌ Ошибок: {data.failed}
            </Typography>
            {data.errors?.length ? (
              <Box component="ul" sx={{ pl: 2, mb: 0 }}>
                {data.errors.slice(0, 3).map((err, i) => (
                  <li key={i}>
                    <Typography variant="caption">{err}</Typography>
                  </li>
                ))}
                {data.errors.length > 3 && (
                  <li>
                    <Typography variant="caption">
                      и ещё {data.errors.length - 3} ошибок…
                    </Typography>
                  </li>
                )}
              </Box>
            ) : null}
          </Alert>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Закрыть</Button>
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
