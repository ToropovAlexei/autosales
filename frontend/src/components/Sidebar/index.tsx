import { Drawer, Box, Divider, Typography } from "@mui/material";
import classes from "./styles.module.css";
import { MenuContent } from "@/components/MenuContent";

export const Sidebar = () => {
  return (
    <Drawer variant="permanent" className={classes.drawer}>
      <Box sx={{ p: 2 }}>
        <Typography variant="h6" align="center">
          Админ панель
        </Typography>
      </Box>
      <Divider />
      <Box
        sx={{
          overflow: "auto",
          height: "100%",
          display: "flex",
          flexDirection: "column",
        }}
      >
        <MenuContent />
      </Box>
    </Drawer>
  );
};
