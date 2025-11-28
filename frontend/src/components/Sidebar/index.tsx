import { Drawer, Box, Button, useMediaQuery, useTheme } from "@mui/material";
import classes from "./styles.module.css";
import { MenuContent } from "@/components/MenuContent";
import { useLogout } from "@/hooks";

interface IProps {
  mobileOpen: boolean;
  toggleMobileDrawer: () => void;
}

export const Sidebar = ({ mobileOpen, toggleMobileDrawer }: IProps) => {
  const logout = useLogout();
  const theme = useTheme();
  const matches = useMediaQuery(theme.breakpoints.up("md"));

  if (matches) {
    return (
      <Drawer variant="permanent" className={classes.drawer}>
        <Box
          sx={{
            overflow: "auto",
            height: "100%",
            display: "flex",
            flexDirection: "column",
            paddingTop: "var(--navbar-height)",
            zIndex: 1,
          }}
        >
          <MenuContent />
          <Button sx={{ mt: "auto" }} onClick={logout}>
            Выход
          </Button>
        </Box>
      </Drawer>
    );
  }

  return (
    <Drawer
      variant="temporary"
      elevation={2}
      open={mobileOpen}
      className={classes.drawer}
      onClose={toggleMobileDrawer}
    >
      <Box
        sx={{
          overflow: "auto",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          zIndex: 1,
        }}
      >
        <MenuContent />
        <Button sx={{ mt: "auto" }} onClick={logout}>
          Выход
        </Button>
      </Box>
    </Drawer>
  );
};
