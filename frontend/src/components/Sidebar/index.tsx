import {
  Button,
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Typography,
} from "@mui/material";
import { usePathname, useRouter } from "next/navigation";
import { MENU_ITEMS } from "./constants";
import { useLogout } from "@/hooks";
import classes from "./styles.module.css";

export const Sidebar = () => {
  const router = useRouter();
  const logout = useLogout();
  const pathname = usePathname();

  return (
    <Drawer variant="permanent" className={classes.drawer}>
      <div className={classes.content}>
        <Typography variant="h6" align="center">
          Меню
        </Typography>
        <List dense disablePadding>
          {MENU_ITEMS.map(({ label, Icon, path }) => (
            <ListItem key={label} disablePadding>
              <ListItemButton
                selected={pathname === path}
                onClick={() => router.push(path)}
              >
                <ListItemIcon>
                  <Icon />
                </ListItemIcon>
                <ListItemText primary={label} />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
        <Button onClick={logout} className={classes.logout}>
          Выйти
        </Button>
      </div>
    </Drawer>
  );
};
