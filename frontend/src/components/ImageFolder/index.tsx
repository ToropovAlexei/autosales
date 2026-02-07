import { useImages, useUploadImage } from "@/hooks";
import { ImageResponse } from "@/types";
import { CircularProgress, IconButton, Typography } from "@mui/material";
import { CONFIG } from "../../../config";
import classes from "./styles.module.css";
import { useRef, useState } from "react";
import { validateImageFile } from "@/utils/validation";
import {
  ENDPOINTS,
  IMAGE_ALLOWED_TYPES,
  INVALID_IMAGE_FILE,
} from "@/constants";
import { toast } from "react-toastify";
import clsx from "clsx";
import CloseIcon from "@mui/icons-material/Close";
import { useMutation } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { ConfirmModal } from "../ConfirmModal";

interface IProps {
  folder: string;
  onSelect?: (image: ImageResponse) => void;
}

export const ImageFolder = ({ onSelect, folder }: IProps) => {
  const [isDragging, setIsDragging] = useState(false);
  const [imageToDelete, setImageToDelete] = useState<ImageResponse | null>(
    null,
  );
  const { data, isLoading } = useImages(folder);
  const { mutate, isPending } = useUploadImage();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleDrop = (event: React.DragEvent<HTMLDivElement>) => {
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

  const { mutate: deleteImage, isPending: isDeletePending } = useMutation({
    mutationFn: (image: ImageResponse) =>
      dataLayer.delete({ url: ENDPOINTS.IMAGES, id: image.id }),
    onError: () => {
      toast.error(
        "Произошла ошибка при удалении изображения. Пожалуйста, попробуйте еще раз.",
      );
    },
    onSuccess: (_, _1, _2, ctx) => {
      ctx.client.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.IMAGES),
      });
      toast.success("Изображение удалено");
    },
  });

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

  const images = data?.data || [];

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file || !validateImageFile(file)) {
      toast.error(INVALID_IMAGE_FILE);
      return;
    }
    mutate({ file, context: folder });
  };

  return (
    <>
      <div
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onClick={() => fileInputRef.current?.click()}
        className={clsx(classes.dropzone, isDragging && classes.dropzoneActive)}
      >
        {isLoading || isPending ? (
          <CircularProgress />
        ) : (
          <>
            {images.length > 0 ? (
              <div className={classes.images}>
                {images.map((image) => (
                  <div
                    key={image.id}
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      onSelect?.(image);
                    }}
                    className={classes.imageItem}
                  >
                    <img src={`${CONFIG.IMAGES_URL}/${image.id}`} />
                    <IconButton
                      className={classes.removeButton}
                      size="small"
                      onClick={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        setImageToDelete(image);
                      }}
                      disabled={isDeletePending}
                    >
                      <CloseIcon fontSize="small" />
                    </IconButton>
                  </div>
                ))}
              </div>
            ) : (
              <Typography>
                Перетащите файлы сюда для загрузки или нажмите для выбора.
              </Typography>
            )}
          </>
        )}
        <input
          type="file"
          ref={fileInputRef}
          onChange={handleFileChange}
          accept={IMAGE_ALLOWED_TYPES.join(",")}
          style={{ display: "none" }}
          multiple={false}
        />
      </div>
      <ConfirmModal
        open={!!imageToDelete}
        contentText="Вы действительно хотите удалить изображение?"
        onClose={() => setImageToDelete(null)}
        onConfirm={() => imageToDelete && deleteImage(imageToDelete)}
        title="Вы уверены?"
        loading={isDeletePending}
      />
    </>
  );
};
