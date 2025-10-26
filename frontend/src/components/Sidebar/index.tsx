import { Drawer, Box, Button } from "@mui/material";
import classes from "./styles.module.css";
import { MenuContent } from "@/components/MenuContent";
import { useLogout } from "@/hooks";

export const Sidebar = () => {
  const logout = useLogout();
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
};
