import * as React from "react";
import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import DashboardRoundedIcon from "@mui/icons-material/DashboardRounded";
import { ColorModeIconDropdown } from "@/components/ColorModeIconDropdown";
import { StoreBalance } from "@/components/StoreBalance";
import classes from "./styles.module.css";

export const NavBar = () => {
  return (
    <AppBar position="fixed" className={classes.appBar}>
      <Toolbar className={classes.toolbar}>
        <div className={classes.logoSection}>
          <DashboardRoundedIcon />
          <Typography variant="h6">Админ панель</Typography>
        </div>
        <StoreBalance />
        <ColorModeIconDropdown />
      </Toolbar>
    </AppBar>
  );
};