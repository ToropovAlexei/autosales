"use client";

import { useState } from "react";
import { useController } from "react-hook-form";
import { IconButton, FormHelperText, Typography } from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { SelectImage } from "@/components/SelectImage";
import { ImageResponse, PermissionName } from "@/types";
import { CONFIG } from "../../../../config";
import classes from "./styles.module.css";
import clsx from "clsx";
import { INVALID_IMAGE_FILE } from "@/constants";
import { toast } from "react-toastify";
import { validateImageFile } from "@/utils/validation";
import { useCan, useUploadImage } from "@/hooks";

interface InputImageProps {
  name: string;
  folder: "other" | "product" | "category" | "fulfillment";
  label?: string;
  disabled?: boolean;
  previewSize?: number;
  alt?: string;
  fullWidth?: boolean;
}

export const InputImage = ({
  name,
  folder,
  label,
  disabled,
  previewSize = 100,
  alt,
  fullWidth,
}: InputImageProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({ name, defaultValue: null });
  const { mutate } = useUploadImage((image) => {
    onChange(image.id);
  });
  const { can: canDelete } = useCan(PermissionName.ImagesDelete);

  const imageId = value || null;

  const handleSelect = (image: ImageResponse) => {
    onChange(image.id);
    setIsOpen(false);
  };

  const handleRemove = () => {
    if (disabled) {
      return;
    }
    onChange(null);
  };

  const handleOpen = () => {
    if (!disabled) {
      setIsOpen(true);
    }
  };

  const handleDrop = (event: React.DragEvent<HTMLDivElement>) => {
    if (disabled) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    setIsDragging(false);
    const file = event.dataTransfer.files?.[0];
    if (!file || !validateImageFile(file)) {
      toast.error(INVALID_IMAGE_FILE);
      return;
    }
    mutate({ file, context: folder });
  };

  const handleDragOver = (event: React.DragEvent<HTMLDivElement>) => {
    if (disabled) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    if (!disabled) {
      setIsDragging(true);
    }
  };

  const handleDragLeave = (event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setIsDragging(false);
  };

  return (
    <div className={clsx(classes.wrapper, fullWidth && classes.fullWidth)}>
      <Typography textAlign="left" width="100%" variant="caption">
        {label || "Изображение"}
      </Typography>
      {imageId ? (
        <div className={classes.preview}>
          <img
            className={classes.img}
            src={`${CONFIG.IMAGES_URL}/${imageId}`}
            alt={alt ?? "Preview"}
            style={{ width: previewSize, height: previewSize }}
          />
          {canDelete && (
            <IconButton
              className={classes.removeButton}
              size="small"
              onClick={handleRemove}
              disabled={disabled}
            >
              <CloseIcon fontSize="small" />
            </IconButton>
          )}
        </div>
      ) : (
        <div
          className={clsx(
            classes.dropzone,
            isDragging && classes.dropzoneActive,
            fullWidth && classes.dropzoneFullWidth,
            disabled && classes.dropzoneDisabled,
          )}
          onClick={handleOpen}
          onDrop={handleDrop}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          role="button"
          tabIndex={disabled ? -1 : 0}
          aria-disabled={disabled}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              handleOpen();
            }
          }}
          style={{
            width: fullWidth ? "100%" : previewSize,
            height: previewSize,
          }}
        >
          Перетащите изображение или нажмите для выбора
        </div>
      )}
      {error && <FormHelperText error>{error.message}</FormHelperText>}
      <SelectImage
        open={isOpen}
        onClose={() => setIsOpen(false)}
        onSelect={handleSelect}
      />
    </div>
  );
};
