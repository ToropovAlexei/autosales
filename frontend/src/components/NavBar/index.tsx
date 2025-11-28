import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import DashboardRoundedIcon from "@mui/icons-material/DashboardRounded";
import { ColorModeIconDropdown } from "@/components/ColorModeIconDropdown";
import { StoreBalance } from "@/components/StoreBalance";
import classes from "./styles.module.css";
import { IconButton, useMediaQuery, useTheme } from "@mui/material";
import MenuIcon from "@mui/icons-material/Menu";

interface IProps {
  toggleMobileDrawer: () => void;
}

export const NavBar = ({ toggleMobileDrawer }: IProps) => {
  const theme = useTheme();
  const matches = useMediaQuery(theme.breakpoints.up("md"));

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
        <StoreBalance />
        <ColorModeIconDropdown />
      </Toolbar>
    </AppBar>
  );
};
