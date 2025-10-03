"use client";

import { SidebarProvider } from "@/components/ui/sidebar";
import { Authorized } from "@/components";
import { Sidebar } from "@/components/Sidebar";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Authorized>
      <SidebarProvider>
        <div className="flex min-h-screen w-full">
          <Sidebar />
          <main className="flex-1 p-8">{children}</main>
        </div>
      </SidebarProvider>
    </Authorized>
  );
}
