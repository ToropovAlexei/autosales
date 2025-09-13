"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/contexts/AuthContext";
import { useEffect } from "react";
import {
  SidebarProvider,
  Sidebar,
  SidebarHeader,
  SidebarContent,
  SidebarFooter,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from "@/components/ui/sidebar";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { isAuthenticated, user, loading, logout } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading && !isAuthenticated) {
      router.push("/login");
    }
  }, [isAuthenticated, loading, router]);

  if (loading || !isAuthenticated) {
    return <div>Loading...</div>; // Or a proper loading spinner
  }

  return (
    <SidebarProvider>
      <div className="flex min-h-screen w-full">
        <Sidebar collapsible="none">
          <SidebarHeader>
            <div className="flex items-center gap-2 font-semibold">
              <span>Меню</span>
            </div>
          </SidebarHeader>
          <SidebarContent>
            <SidebarMenu>
              {user?.role === "admin" && (
                <SidebarMenuItem>
                  <SidebarMenuButton asChild>
                    <Link href="/dashboard">Дашборд</Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              )}
              <SidebarMenuItem>
                <SidebarMenuButton asChild>
                  <Link href="/categories">Категории</Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton asChild>
                  <Link href="/products">Товары</Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              {user?.role === "admin" && (
                <>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild>
                      <Link href="/bot-users">Пользователи бота</Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild>
                      <Link href="/transactions">Транзакции</Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild>
                      <Link href="/orders">Заказы</Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild>
                      <Link href="/stock">Склад</Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                </>
              )}
              {(user?.role === "admin" || user?.role === "seller") && (
                <SidebarMenuItem>
                  <SidebarMenuButton asChild>
                    <Link href="/referral-bots">Реферальные боты</Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              )}
              <SidebarMenuItem>
                <SidebarMenuButton asChild>
                  <Link href="/settings">Настройки</Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarContent>
          <SidebarFooter>
            <Button variant="outline" onClick={logout}>
              Выход
            </Button>
          </SidebarFooter>
        </Sidebar>
        <main className="flex-1 p-8">{children}</main>
      </div>
    </SidebarProvider>
  );
}
