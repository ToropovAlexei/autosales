"use client";

import { Authorized, Sidebar } from "@/components";
import classes from "./styles.module.css";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Authorized>
      <div className={classes.container}>
        <Sidebar />
        <main className={classes.main}>{children}</main>
      </div>
    </Authorized>
  );
}
