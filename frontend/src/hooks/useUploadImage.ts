import { ENDPOINTS } from "@/constants";
import { dataLayer } from "@/lib/dataLayer";
import { ImageResponse } from "@/types";
import { queryKeys } from "@/utils/query";
import { useMutation } from "@tanstack/react-query";
import { toast } from "react-toastify";

export const useUploadImage = (onSuccess?: (image: ImageResponse) => void) =>
  useMutation({
    mutationFn: (variables: { file: File; context: string }) => {
      const formData = new FormData();
      formData.append("file", variables.file);
      formData.append("context", variables.context);
      return dataLayer.create<ImageResponse>({
        url: ENDPOINTS.IMAGES,
        params: formData,
      });
    },
    onSuccess: (data, _1, _2, ctx) => {
      ctx.client.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.IMAGES),
      });
      toast.success("Изображение загружено");
      onSuccess?.(data);
    },
    onError: (err) => {
      toast.error(
        "Произошла ошибка при загрузке изображения. Пожалуйста, попробуйте еще раз.",
      );
    },
  });
