import * as React from "react";
import Button from "@mui/material/Button";
import Divider from "@mui/material/Divider";
import Drawer from "@mui/material/Drawer";
import Stack from "@mui/material/Stack";
import LogoutRoundedIcon from "@mui/icons-material/LogoutRounded";
import { MenuContent } from "@/components/MenuContent";
import classes from "./styles.module.css";

interface SideMenuMobileProps {
  open: boolean | undefined;
  toggleDrawer: (newOpen: boolean) => () => void;
}

export const SideMenuMobile = ({ open, toggleDrawer }: SideMenuMobileProps) => {
  return (
    <Drawer
      anchor="right"
      open={open}
      onClose={toggleDrawer(false)}
      sx={{
        zIndex: (theme) => theme.zIndex.drawer + 1,
      }}
      slotProps={{
        paper: { className: classes.drawerPaper },
      }}
    >
      <Stack className={classes.stackRoot}>
        <Stack sx={{ flexGrow: 1 }}>
          <MenuContent />
          <Divider />
        </Stack>
        <Stack sx={{ p: 2 }}>
          <Button
            variant="outlined"
            fullWidth
            startIcon={<LogoutRoundedIcon />}
          >
            Logout
          </Button>
        </Stack>
      </Stack>
    </Drawer>
  );
};
