import * as React from 'react';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import Stack from '@mui/material/Stack';
import { usePathname, useRouter } from "next/navigation";
import { MENU_ITEMS } from '@/components/Sidebar/constants';
import classes from './styles.module.css';

export const MenuContent = () => {
  const router = useRouter();
  const pathname = usePathname();

  return (
    <Stack className={classes.menuContentStack}>
      <List dense>
        {MENU_ITEMS.map(({ label, Icon, path }) => (
          <ListItem key={label} disablePadding sx={{ display: 'block' }}>
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
    </Stack>
  );
}