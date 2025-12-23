"use client";

import { useState } from "react";
import { Button } from "@mui/material";
import { SelectImage } from "@/components/SelectImage"; // The detailed modal
import { IImage } from "@/types";
import { CONFIG } from "../../../config";
import classes from "./styles.module.css";

interface ImageSelectProps {
  selectedImageId: string | null;
  onSelectImage: (id: string | null) => void;
}

export const ImageSelect = ({ selectedImageId, onSelectImage }: ImageSelectProps) => {
  const [isModalOpen, setIsModalOpen] = useState(false);

  const handleImageSelect = (image: IImage) => {
    onSelectImage(image.ID);
    setIsModalOpen(false);
  };

  return (
    <>
      <div className={classes.selectImg}>
        <Button variant="outlined" onClick={() => setIsModalOpen(true)}>
          {selectedImageId ? "Изменить изображение" : "Выбрать изображение"}
        </Button>
        {selectedImageId && (
          <img
            className={classes.img}
            src={`${CONFIG.IMAGES_URL}/${selectedImageId}`}
            alt="Selected"
            width="30%"
          />
        )}
      </div>

      <SelectImage
        open={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSelect={handleImageSelect}
      />
    </>
  );
};
