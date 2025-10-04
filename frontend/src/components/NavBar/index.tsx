import * as React from "react";
import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import DashboardRoundedIcon from "@mui/icons-material/DashboardRounded";
import { ColorModeIconDropdown } from "@/components/ColorModeIconDropdown";
import classes from "./styles.module.css";

export const NavBar = () => {
  return (
    <AppBar position="fixed" className={classes.appBar}>
      <Toolbar className={classes.toolbar}>
        <Stack direction="row" className={classes.logoSection}>
          <DashboardRoundedIcon className={classes.logoIcon} />
          <Typography variant="h6" component="div" color="text.primary">
            Админ панель
          </Typography>
        </Stack>
        <Stack direction="row" spacing={1}>
          <ColorModeIconDropdown />
        </Stack>
      </Toolbar>
    </AppBar>
  );
};
