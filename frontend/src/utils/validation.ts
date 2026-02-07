import { IMAGE_ALLOWED_TYPES } from "@/constants";

export const validateImageFile = (file: File) =>
  IMAGE_ALLOWED_TYPES.includes(file.type);
