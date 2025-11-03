import { Button } from "@mui/material";
import { ChangeEventHandler, PropsWithChildren, useRef } from "react";
import CloudUploadIcon from "@mui/icons-material/CloudUpload";

interface IProps extends PropsWithChildren {
  loading?: boolean;
  accept?: string;
  multiple?: boolean;
  disabled?: boolean;
  onFileChange?: ChangeEventHandler<HTMLInputElement>;
}

export const UploadBtn = ({
  onFileChange,
  loading,
  accept,
  multiple,
  disabled,
  children,
}: IProps) => {
  const ref = useRef<HTMLInputElement>(null);
  const handleUploadClick = () => ref.current?.click();
  return (
    <Button
      variant="contained"
      onClick={handleUploadClick}
      disabled={disabled}
      loading={loading}
      startIcon={<CloudUploadIcon />}
    >
      {children}
      <input
        type="file"
        ref={ref}
        onChange={(e) => {
          onFileChange?.(e);
          e.target.value = "";
        }}
        accept={accept}
        style={{ display: "none" }}
        multiple={multiple}
      />
    </Button>
  );
};
