import { Drawer, Box } from "@mui/material";
import classes from "./styles.module.css";
import { MenuContent } from "@/components/MenuContent";

export const Sidebar = () => {
  return (
    <Drawer variant="permanent" className={classes.drawer}>
      <Box
        sx={{
          overflow: "auto",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          paddingTop: "var(--navbar-height)",
          zIndex: 1,
        }}
      >
        <MenuContent />
      </Box>
    </Drawer>
  );
};
