"use client";

import { FormProvider, useForm } from "react-hook-form";
import { AdapterDayjs } from "@mui/x-date-pickers/AdapterDayjs";
import { LocalizationProvider } from "@mui/x-date-pickers/LocalizationProvider";
import { PageLayout } from "@/components/PageLayout";
import { BroadcastForm } from "./types";
import { InputMsg } from "./components/InputMsg";
import { Card, CardContent } from "@mui/material";
import { Filters } from "./components/Filters";
import { UsersTable } from "./components/UsersTable";
import classes from "./styles.module.css";

export default function BroadcastsPage() {
  const form = useForm<BroadcastForm>();

  return (
    <LocalizationProvider dateAdapter={AdapterDayjs}>
      <PageLayout title="Управление рекламой">
        <Card>
          <CardContent>
            <FormProvider {...form}>
              <div className={classes.container}>
                <div className={classes.inputs}>
                  <InputMsg />
                  <Filters />
                </div>
                <UsersTable />
              </div>
            </FormProvider>
          </CardContent>
        </Card>
      </PageLayout>
    </LocalizationProvider>
  );
}
