"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { usePermissions } from "@/hooks";
import { MENU_ITEMS } from "@/components/Sidebar/constants";
import { CircularProgress, Box } from "@mui/material";

export default function PostLoginPage() {
  const { permissions, isLoading } = usePermissions();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading && permissions.length > 0) {
      const firstAccessiblePage = MENU_ITEMS.find(item => permissions.includes(item.permission));
      
      if (firstAccessiblePage) {
        router.push(firstAccessiblePage.path);
      } else {
        // Fallback if no permissions match the menu items
        // Maybe redirect to a dedicated "no access" page or back to login with an error
        router.push("/login?error=no_access");
      }
    }
  }, [isLoading, permissions, router]);

  return (
    <Box
      sx={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "100vh",
      }}
    >
      <CircularProgress />
    </Box>
  );
}
