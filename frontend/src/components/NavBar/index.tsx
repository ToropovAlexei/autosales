import * as React from 'react';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Stack from '@mui/material/Stack';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import DashboardRoundedIcon from '@mui/icons-material/DashboardRounded';
import { ColorModeIconDropdown } from '@/components/ColorModeIconDropdown';

export const NavBar = () => {
  return (
    <AppBar
      position="fixed"
      sx={{
        boxShadow: 0,
        bgcolor: 'background.paper',
        backgroundImage: 'none',
        borderBottom: '1px solid',
        borderColor: 'divider',
        zIndex: (theme) => theme.zIndex.drawer + 1,
      }}
    >
      <Toolbar
        sx={{
          height: 'var(--template-navbar-height)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          gap: 1,
        }}
      >
        <Stack direction="row" spacing={1} alignItems="center">
          <DashboardRoundedIcon color="primary" />
          <Typography variant="h6" component="div">
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