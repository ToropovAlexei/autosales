"use client";

import { Authorized, Sidebar } from "@/components";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Authorized>
      <div className="flex min-h-screen w-full">
        <Sidebar />
        <main className="flex-1 p-8">{children}</main>
      </div>
    </Authorized>
  );
}
