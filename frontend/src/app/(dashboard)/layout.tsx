"use client";

import { Authorized, Sidebar } from "@/components";
import { Box } from "@mui/material";
import { alpha } from "@mui/material/styles";
import { NavBar } from "@/components/NavBar";
import classes from "./styles.module.css";
import { useState } from "react";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [isMobileDrawerOpen, setMobileDrawerOpen] = useState(false);

  const handleToggleMobileDrawer = () => setMobileDrawerOpen((p) => !p);

  return (
    <Authorized>
      <Box sx={{ display: "flex" }}>
        <NavBar toggleMobileDrawer={handleToggleMobileDrawer} />
        <Sidebar
          mobileOpen={isMobileDrawerOpen}
          toggleMobileDrawer={handleToggleMobileDrawer}
        />
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
