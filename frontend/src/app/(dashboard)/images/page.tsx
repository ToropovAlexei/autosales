"use client";

import React, { useState, useRef } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useList } from "@/hooks";
import { dataLayer } from "@/lib/dataLayer";
import { PageLayout } from "@/components/PageLayout";
import {
  Button,
  ImageList,
  ImageListItem,
  CircularProgress,
  Alert,
  Box,
  Typography,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
} from "@mui/material";
import { ENDPOINTS } from "@/constants";
import classes from "./styles.module.css";
import { CONFIG } from "../../../../config";
import { queryKeys } from "@/utils/query";

interface IImage {
  ID: string;
  OriginalFilename: string;
}

const FOLDERS = [
  { id: "product_images", name: "Изображения товаров" },
  { id: "fulfillment_images", name: "Выдача (картинки)" },
  { id: "categories", name: "Категории" },
];
const ALLOWED_TYPES = ["image/jpeg", "image/png", "image/gif", "image/webp"];

export default function ImagesPage() {
  const [error, setError] = useState<string | null>(null);
  const [selectedFolder, setSelectedFolder] = useState("categories");
  const [isDragging, setIsDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const queryClient = useQueryClient();

  const { data: imagesData, isPending: isLoadingImages } = useList<IImage>({
    endpoint: ENDPOINTS.IMAGES,
    filter: { folder: selectedFolder },
  });
  const images = imagesData?.data || [];

  const uploadMutation = useMutation({
    mutationFn: (variables: { file: File; folder: string }) => {
      const formData = new FormData();
      formData.append("image", variables.file);
      formData.append("folder", variables.folder);
      return dataLayer.create<{ data: IImage }>({
        url: ENDPOINTS.IMAGES,
        params: formData,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.IMAGES),
      });
      setError(null);
    },
    onError: (err) => {
      setError(err.message || "Failed to upload image.");
    },
  });

  const validateFile = (file: File) => {
    if (!ALLOWED_TYPES.includes(file.type)) {
      setError(
        `Неверный тип файла. Пожалуйста, загрузите изображение в формате JPEG, PNG, GIF или WEBP.`
      );
      return false;
    }
    setError(null);
    return true;
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file && validateFile(file)) {
      uploadMutation.mutate({ file, folder: selectedFolder });
    }
  };

  const handleUploadClick = () => fileInputRef.current?.click();

  const handleDrop = (event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setIsDragging(false);
    const file = event.dataTransfer.files?.[0];
    if (file && validateFile(file)) {
      uploadMutation.mutate({ file, folder: selectedFolder });
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
    <PageLayout title="Управление изображениями">
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
          <Box mb={2}>
            <Button
              variant="contained"
              onClick={handleUploadClick}
              disabled={uploadMutation.isPending}
            >
              {uploadMutation.isPending ? (
                <CircularProgress size={24} />
              ) : (
                `Загрузить в "${
                  FOLDERS.find((f) => f.id === selectedFolder)?.name
                }"`
              )}
            </Button>
            <input
              type="file"
              ref={fileInputRef}
              onChange={handleFileChange}
              accept={ALLOWED_TYPES.join(",")}
              style={{ display: "none" }}
              multiple={false}
            />
          </Box>

          {error && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {error}
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
            {isLoadingImages ? (
              <CircularProgress />
            ) : images.length > 0 ? (
              <ImageList variant="quilted" cols={8} gap={8}>
                {images.map((image) => (
                  <ImageListItem key={image.ID}>
                    <img
                      className={classes.img}
                      src={`${CONFIG.IMAGES_URL}/${image.ID}`}
                      alt={image.OriginalFilename}
                    />
                  </ImageListItem>
                ))}
              </ImageList>
            ) : (
              <Typography>
                Перетащите файлы сюда или используйте кнопку для загрузки.
              </Typography>
            )}
          </div>
        </div>
      </div>
    </PageLayout>
  );
}
