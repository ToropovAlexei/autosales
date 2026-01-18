import { ConfirmModal, InputText, SelectImage } from "@/components";
import { useController, useFormContext, useWatch } from "react-hook-form";
import { BroadcastForm } from "../../types";
import { useState } from "react";
import { Button, Typography } from "@mui/material";
import classes from "./styles.module.css";
import { CONFIG } from "../../../../../../config";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { Broadcast, ImageResponse, NewBroadcast } from "@/types";
import { useMutation } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { formToFilters } from "../../utils";

export const InputMsg = () => {
  const [isImageSelectorOpen, setIsImageSelectorOpen] = useState(false);
  const [isConfirmModalOpen, setIsConfirmModalOpen] = useState(false);
  const {
    field: { value, onChange },
  } = useController<BroadcastForm>({ name: "image_id" });

  const handleImageSelect = (image: ImageResponse) => {
    onChange(image.id);
    setIsImageSelectorOpen(false);
  };

  const { mutate, isPending } = useMutation<Broadcast, Error, NewBroadcast>({
    mutationFn: (params) =>
      dataLayer.create({ url: ENDPOINTS.BROADCAST, params }),
    onSuccess: () => {
      toast.success("Рекламное сообщение отправлено");
    },
    onError: () => {
      toast.error("Произошла ошибка");
    },
  });
  const { getValues } = useFormContext<BroadcastForm>();
  const [imageId, text] = useWatch<BroadcastForm>({
    name: ["image_id", "text"],
  });
  const isAbleToSend = !!imageId || !!(text && String(text).trim());

  const handleConfirm = () => {
    const { image_id, text, ...filters } = getValues();
    mutate({
      content_image_id: image_id || undefined,
      content_text: text,
      filters: { filters: formToFilters(filters) },
    });
  };

  return (
    <div>
      <Typography variant="h6">Рекламное сообщение</Typography>
      <InputText
        name="text"
        label="Сообщение"
        minRows={3}
        multiline
        fullWidth
      />
      <div className={classes.selectImg}>
        <Button variant="outlined" onClick={() => setIsImageSelectorOpen(true)}>
          Выбрать изображение
        </Button>
        {value && (
          <img
            className={classes.img}
            src={`${CONFIG.IMAGES_URL}/${value}`}
            alt="Preview"
            width="30%"
          />
        )}
      </div>
      <SelectImage
        open={isImageSelectorOpen}
        onClose={() => setIsImageSelectorOpen(false)}
        onSelect={handleImageSelect}
      />
      <div className={classes.send}>
        <Button
          variant="contained"
          onClick={() => setIsConfirmModalOpen(true)}
          loading={isPending}
          disabled={!isAbleToSend}
        >
          Сделать рекламную рассылку
        </Button>
      </div>
      <ConfirmModal
        onClose={() => setIsConfirmModalOpen(false)}
        open={isConfirmModalOpen}
        title="Вы уверены?"
        contentText="Вы уверены, что хотите сделать рекламную рассылку?"
        onConfirm={handleConfirm}
        confirmBtnText="Отправить"
        loading={isPending}
      />
    </div>
  );
};
