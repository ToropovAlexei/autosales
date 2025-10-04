import {
  Button,
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
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>{title}</DialogTitle>
      <DialogContent>
        <DialogContentText>{contentText}</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button variant="contained" onClick={handleConfirm} loading={loading}>
          {confirmBtnText}
        </Button>
        <Button variant="outlined" onClick={onClose} loading={loading}>
          {closeBtnText}
        </Button>
      </DialogActions>
    </Dialog>
  );
};
