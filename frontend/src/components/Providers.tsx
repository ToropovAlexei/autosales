"use client";

import { useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { CssBaseline, ThemeProvider } from "@mui/material";
import { LocalizationProvider } from "@mui/x-date-pickers";
import { AdapterDayjs } from "@mui/x-date-pickers/AdapterDayjs";
import { ToastContainer } from "react-toastify";
import { ruRU } from "@mui/x-date-pickers/locales";
import { createAppTheme } from "@/themes";
import relativeTime from "dayjs/plugin/relativeTime";
import "dayjs/locale/ru";
import dayjs from "dayjs";

dayjs.extend(relativeTime);
dayjs.locale("ru");

const theme = createAppTheme();

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 2 * 60 * 1000,
            refetchInterval: 2 * 60 * 1000,
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
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </ThemeProvider>
    </LocalizationProvider>
  );
}
