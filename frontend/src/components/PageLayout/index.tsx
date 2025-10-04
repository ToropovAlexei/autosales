import * as React from 'react';
import { Typography } from '@mui/material';
import classes from './styles.module.css';

interface PageLayoutProps {
  title: string;
  children: React.ReactNode;
}

export const PageLayout = ({ title, children }: PageLayoutProps) => {
  return (
    <div className={classes.page}>
      <Typography variant="h4" gutterBottom>
        {title}
      </Typography>
      {children}
    </div>
  );
};
