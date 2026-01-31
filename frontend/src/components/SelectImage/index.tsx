"use client";

import React, { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useList } from "@/hooks";
import { dataLayer } from "@/lib/dataLayer";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  ImageList,
  ImageListItem,
  CircularProgress,
  Alert,
  Typography,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  DialogActions,
  Button,
} from "@mui/material";
import { ENDPOINTS } from "@/constants";
import classes from "./styles.module.css";
import { CONFIG } from "../../../config";
import { queryKeys } from "@/utils/query";
import { ImageResponse } from "@/types/image";

const FOLDERS = [
  { id: "product", name: "Изображения товаров" },
  { id: "fulfillment", name: "Выдача (картинки)" },
  { id: "category", name: "Категории" },
];
const ALLOWED_TYPES = ["image/jpeg", "image/png", "image/gif", "image/webp"];

interface SelectImageProps {
  open: boolean;
  onClose: () => void;
  onSelect: (image: ImageResponse) => void;
}

export const SelectImage = ({ open, onClose, onSelect }: SelectImageProps) => {
  const [selectedFolder, setSelectedFolder] = useState(FOLDERS[0].id);
  const [isDragging, setIsDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const client = useQueryClient();

  const { data: imagesData, isPending: isLoadingImages } =
    useList<ImageResponse>({
      endpoint: ENDPOINTS.IMAGES,
      filter: {
        filters: [{ op: "eq", field: "context", value: selectedFolder }],
      },
      enabled: open,
    });
  const images = imagesData?.data || [];

  const uploadMutation = useMutation({
    mutationFn: (variables: { file: File; context: string }) => {
      const formData = new FormData();
      formData.append("file", variables.file);
      formData.append("context", variables.context);
      return dataLayer.create<{ data: ImageResponse }>({
        url: ENDPOINTS.IMAGES,
        params: formData,
      });
    },
    onSuccess: () => {
      client.invalidateQueries({ queryKey: queryKeys.list(ENDPOINTS.IMAGES) });
      setError(null);
    },
    onError: (err) => {
      setError(err.message || "Failed to upload image.");
    },
  });

  const validateFile = (file: File) => {
    if (!ALLOWED_TYPES.includes(file.type)) {
      setError(
        `Неверный тип файла. Пожалуйста, загрузите изображение в формате JPEG, PNG, GIF или WEBP.`,
      );
      return false;
    }
    setError(null);
    return true;
  };

  const handleDrop = (event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setIsDragging(false);
    const file = event.dataTransfer.files?.[0];
    if (file && validateFile(file)) {
      uploadMutation.mutate({ file, context: selectedFolder });
    }
  };

  const handleDragOver = (event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setIsDragging(false);
  };

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="lg">
      <DialogTitle>Выберите изображение</DialogTitle>
      <DialogContent>
        <div className={classes.container}>
          <div className={classes.sidebar}>
            <Typography variant="h6" gutterBottom>
              Папки
            </Typography>
            <List component="nav">
              {FOLDERS.map((folder) => (
                <ListItem key={folder.id} disablePadding>
                  <ListItemButton
                    selected={selectedFolder === folder.id}
                    onClick={() => setSelectedFolder(folder.id)}
                  >
                    <ListItemText primary={folder.name} />
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
          </div>
          <div className={classes.mainContent}>
            {error && (
              <Alert severity="error" sx={{ mb: 2 }}>
                {error}
              </Alert>
            )}
            {uploadMutation.error && (
              <Alert severity="error" sx={{ mb: 2 }}>
                {uploadMutation.error.message}
              </Alert>
            )}
            <div
              onDrop={handleDrop}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              className={`${classes.dropzone} ${
                isDragging ? classes.dropzoneActive : ""
              }`}
            >
              {isLoadingImages || uploadMutation.isPending ? (
                <CircularProgress />
              ) : images.length > 0 ? (
                <ImageList cols={8}>
                  {images.map((image) => (
                    <ImageListItem
                      key={image.id}
                      onClick={() => onSelect(image)}
                      sx={{ cursor: "pointer" }}
                    >
                      <img src={`${CONFIG.IMAGES_URL}/${image.id}`} />
                    </ImageListItem>
                  ))}
                </ImageList>
              ) : (
                <Typography color="info">
                  Перетащите файлы сюда для загрузки.
                </Typography>
              )}
            </div>
          </div>
        </div>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Закрыть</Button>
      </DialogActions>
    </Dialog>
  );
};
