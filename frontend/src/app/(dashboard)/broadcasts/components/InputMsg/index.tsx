import { ConfirmModal, InputImage, InputText } from "@/components";
import { useFormContext, useWatch } from "react-hook-form";
import { BroadcastForm } from "../../types";
import { useState } from "react";
import { Button, Typography } from "@mui/material";
import classes from "./styles.module.css";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { Broadcast, NewBroadcast } from "@/types";
import { useMutation } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { formToFilters } from "../../utils";

export const InputMsg = () => {
  const [isConfirmModalOpen, setIsConfirmModalOpen] = useState(false);

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
      <div className={classes.inputMsgContainer}>
        <InputText
          name="text"
          label="Сообщение"
          minRows={3}
          multiline
          fullWidth
        />
        <InputImage
          name="image_id"
          buttonLabel="Выбрать изображение"
          fullWidth
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
