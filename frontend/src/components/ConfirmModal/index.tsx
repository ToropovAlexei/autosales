import {
  Button,
  ButtonOwnProps,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
} from "@mui/material";
import { MouseEvent, useCallback } from "react";

interface IProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (() => void) | ((event: MouseEvent<HTMLButtonElement>) => void);
  contentText: string;
  title: string;
  loading?: boolean;
  confirmBtnText?: string;
  closeBtnText?: string;
  confirmBtnColor?: ButtonOwnProps["color"];
  closeBtnColor?: ButtonOwnProps["color"];
  preventCloseOnConfirm?: boolean;
}

export const ConfirmModal = ({
  open,
  onClose,
  onConfirm,
  contentText,
  title,
  confirmBtnText = "Ок",
  closeBtnText = "Закрыть",
  loading,
  preventCloseOnConfirm,
  closeBtnColor,
  confirmBtnColor,
}: IProps) => {
  const handleConfirm = useCallback(
    (event: MouseEvent<HTMLButtonElement>) => {
      onConfirm(event);
      if (!preventCloseOnConfirm) {
        onClose();
      }
    },
    [onClose, onConfirm, preventCloseOnConfirm]
  );

  return (
    <Dialog open={open} onClose={onClose} disableScrollLock>
      <DialogTitle>{title}</DialogTitle>
      <DialogContent>
        <DialogContentText>{contentText}</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button
          variant="contained"
          color={confirmBtnColor}
          onClick={handleConfirm}
          loading={loading}
        >
          {confirmBtnText}
        </Button>
        <Button
          variant="outlined"
          color={closeBtnColor}
          onClick={onClose}
          loading={loading}
        >
          {closeBtnText}
        </Button>
      </DialogActions>
    </Dialog>
  );
};
