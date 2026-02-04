"use client";

import { useState } from "react";
import { useController } from "react-hook-form";
import { Box, Button, IconButton, FormHelperText } from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { SelectImage } from "@/components/SelectImage";
import { ImageResponse } from "@/types";
import { CONFIG } from "../../../../config";
import classes from "./styles.module.css";
import clsx from "clsx";

interface InputImageProps {
  name: string;
  buttonLabel?: string;
  disabled?: boolean;
  previewSize?: number;
  alt?: string;
  fullWidth?: boolean;
}

export const InputImage = ({
  name,
  buttonLabel,
  disabled,
  previewSize = 100,
  alt,
  fullWidth,
}: InputImageProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({ name, defaultValue: null });

  const imageId = value || null;

  const handleSelect = (image: ImageResponse) => {
    onChange(image.id);
    setIsOpen(false);
  };

  const handleRemove = () => {
    onChange(null);
  };

  return (
    <div className={clsx(classes.wrapper, fullWidth && classes.fullWidth)}>
      {imageId && (
        <div className={classes.preview}>
          <img
            className={classes.img}
            src={`${CONFIG.IMAGES_URL}/${imageId}`}
            alt={alt ?? "Preview"}
            style={{ width: previewSize, height: previewSize }}
          />
          <IconButton
            className={classes.removeButton}
            size="small"
            onClick={handleRemove}
            disabled={disabled}
          >
            <CloseIcon fontSize="small" />
          </IconButton>
        </div>
      )}
      <Button
        variant="outlined"
        onClick={() => setIsOpen(true)}
        disabled={disabled}
      >
        {buttonLabel ?? "Выбрать изображение"}
      </Button>
      {error && <FormHelperText error>{error.message}</FormHelperText>}
      <SelectImage
        open={isOpen}
        onClose={() => setIsOpen(false)}
        onSelect={handleSelect}
      />
    </div>
  );
};
