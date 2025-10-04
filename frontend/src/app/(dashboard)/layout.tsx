'use client';

import { Authorized, Sidebar } from "@/components";
import { Box } from "@mui/material";
import { alpha } from "@mui/material/styles";
import { NavBar } from '@/components/NavBar';
import classes from './styles.module.css';

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Authorized>
      <Box sx={{ display: "flex" }}>
        <NavBar />
        <Sidebar />
        <Box
          component="main"
          className={classes.mainContent}
          sx={(theme) => ({
            backgroundColor: theme.vars
              ? `rgba(${theme.vars.palette.background.defaultChannel} / 1)`
              : alpha(theme.palette.background.default, 1),
          })}
        >
          {children}
        </Box>
      </Box>
    </Authorized>
  );
}
