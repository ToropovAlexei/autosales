import { ENDPOINTS } from "@/constants";
import { ImageResponse } from "@/types";
import { useList } from "./useList";

export const useImages = (folder: string, enabled = true) =>
  useList<ImageResponse>({
    endpoint: ENDPOINTS.IMAGES,
    filter: {
      filters: [{ op: "eq", field: "context", value: folder }],
    },
    enabled,
  });
