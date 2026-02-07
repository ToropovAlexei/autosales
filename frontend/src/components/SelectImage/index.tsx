"use client";

import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
} from "@mui/material";
import { ImageResponse } from "@/types/image";
import { ImageFoldersManager } from "../ImageFoldersManager";

interface SelectImageProps {
  open: boolean;
  onClose: () => void;
  onSelect: (image: ImageResponse) => void;
}

export const SelectImage = ({ open, onClose, onSelect }: SelectImageProps) => {
  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="lg">
      <DialogTitle>Выберите изображение</DialogTitle>
      <DialogContent>
        <ImageFoldersManager onSelect={onSelect} />
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Закрыть</Button>
      </DialogActions>
    </Dialog>
  );
};
