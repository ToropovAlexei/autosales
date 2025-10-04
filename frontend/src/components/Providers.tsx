'use client';

import { useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { CssBaseline, ThemeProvider } from "@mui/material";
import { Toaster } from "sonner";
import { LocalizationProvider } from "@mui/x-date-pickers";
import { AdapterDayjs } from "@mui/x-date-pickers/AdapterDayjs";
import { ToastContainer } from "react-toastify";
import { ruRU } from "@mui/x-date-pickers/locales";
import { createAppTheme } from "@/themes";
import { chartsCustomizations, dataGridCustomizations, datePickersCustomizations, treeViewCustomizations } from '@/../dashboard/theme/customizations';

const xThemeComponents = {
  ...chartsCustomizations,
  ...dataGridCustomizations,
  ...datePickersCustomizations,
  ...treeViewCustomizations,
};

const theme = createAppTheme(xThemeComponents);

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 1000 * 60,
          },
        },
      })
  );

  return (
    <LocalizationProvider
      dateAdapter={AdapterDayjs}
      localeText={
        ruRU.components.MuiLocalizationProvider.defaultProps.localeText
      }
    >
      <ThemeProvider theme={theme}>
        <CssBaseline enableColorScheme />
        <ToastContainer />
        <Toaster />
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </ThemeProvider>
    </LocalizationProvider>
  );
}