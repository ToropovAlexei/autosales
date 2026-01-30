import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import DashboardRoundedIcon from "@mui/icons-material/DashboardRounded";
import { ColorModeIconDropdown } from "@/components/ColorModeIconDropdown";
import { ConfirmModal } from "@/components/ConfirmModal";
import { StoreBalance } from "@/components/StoreBalance";
import { ENDPOINTS } from "@/constants";
import { api } from "@/lib/api";
import classes from "./styles.module.css";
import { Button, IconButton, useMediaQuery, useTheme } from "@mui/material";
import MenuIcon from "@mui/icons-material/Menu";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { toast } from "react-toastify";

interface IProps {
  toggleMobileDrawer: () => void;
}

export const NavBar = ({ toggleMobileDrawer }: IProps) => {
  const theme = useTheme();
  const matches = useMediaQuery(theme.breakpoints.up("md"));
  const client = useQueryClient();
  const [confirmOpen, setConfirmOpen] = useState(false);
  const resetMutation = useMutation({
    mutationFn: () => api.post(ENDPOINTS.DEV_RESET_DATA).json(),
    onSuccess: () => {
      client.invalidateQueries();
      toast.success("Тестовые данные очищены");
    },
    onError: () => toast.error("Не удалось очистить тестовые данные"),
  });

  return (
    <AppBar position="fixed" className={classes.appBar}>
      <Toolbar className={classes.toolbar}>
        {matches ? (
          <div className={classes.logoSection}>
            <DashboardRoundedIcon />
            <Typography variant="h6">Админ панель</Typography>
          </div>
        ) : (
          <IconButton onClick={toggleMobileDrawer}>
            <MenuIcon />
          </IconButton>
        )}
        <div className={classes.actions}>
          <>
            <Button
              variant="outlined"
              color="error"
              size="small"
              onClick={() => setConfirmOpen(true)}
              disabled={resetMutation.isPending}
            >
              Сбросить тестовые данные
            </Button>
            <ConfirmModal
              open={confirmOpen}
              onClose={() => setConfirmOpen(false)}
              onConfirm={() => resetMutation.mutate()}
              title="Сбросить тестовые данные?"
              contentText="Будут удалены товары, заказы, подписки, категории, изображения, покупатели, движения на складе, транзакции, инвойсы, рассылки и аудит логи. Настройки, администраторы, токены и боты останутся."
              confirmBtnText="Сбросить"
              confirmBtnColor="error"
              loading={resetMutation.isPending}
            />
          </>
          <StoreBalance />
          <ColorModeIconDropdown />
        </div>
      </Toolbar>
    </AppBar>
  );
};
