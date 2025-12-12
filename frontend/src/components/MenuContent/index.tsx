import {
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
} from "@mui/material";
import { usePathname, useRouter } from "next/navigation";
import { MENU_ITEMS } from "@/components/Sidebar/constants";
import { useCan } from "@/hooks";
import { ROUTES_ACCESS_MAP } from "@/constants/access";
import { APP_ROUTES } from "@/constants/routing";

const MenuItem = ({ item }: { item: (typeof MENU_ITEMS)[number] }) => {
  const router = useRouter();
  const pathname = usePathname();
  const { label, Icon, route } = item;
  const { can: canAccess } = useCan(ROUTES_ACCESS_MAP[route]);

  if (!canAccess) {
    return null;
  }

  return (
    <ListItem key={route} disablePadding>
      <ListItemButton
        selected={pathname.startsWith(APP_ROUTES[route])}
        onClick={() => router.push(APP_ROUTES[route])}
      >
        <ListItemIcon>
          <Icon />
        </ListItemIcon>
        <ListItemText primary={label} />
      </ListItemButton>
    </ListItem>
  );
};

export const MenuContent = () => {
  return (
    <List>
      {MENU_ITEMS.map((item) => (
        <MenuItem key={item.route} item={item} />
      ))}
    </List>
  );
};
