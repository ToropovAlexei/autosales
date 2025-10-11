import { List, ListItem, ListItemButton, ListItemIcon, ListItemText } from "@mui/material";
import { usePathname, useRouter } from "next/navigation";
import { MENU_ITEMS } from '@/components/Sidebar/constants';
import { useCan } from "@/hooks";

const MenuItem = ({ item }) => {
  const router = useRouter();
  const pathname = usePathname();
  const { label, Icon, path, permission } = item;
  const canAccess = useCan(permission);

  if (!canAccess) {
    return null;
  }

  return (
    <ListItem key={path} disablePadding>
      <ListItemButton
        selected={pathname.startsWith(path)}
        onClick={() => router.push(path)}
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
        <MenuItem key={item.path} item={item} />
      ))}
    </List>
  );
};
