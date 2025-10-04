import * as React from "react";
import { Typography } from "@mui/material";
import classes from "./styles.module.css";

interface PageLayoutProps {
  title: string;
  children: React.ReactNode;
}

export const PageLayout = ({ title, children }: PageLayoutProps) => {
  return (
    <div className={classes.page}>
      <Typography variant="h4" lineHeight={1} mb={2}>
        {title}
      </Typography>
      {children}
    </div>
  );
};
