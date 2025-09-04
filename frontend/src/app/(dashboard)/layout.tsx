
'use client';

import Link from "next/link";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/contexts/AuthContext";
import { useEffect } from "react";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { isAuthenticated, user, loading, logout } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading && !isAuthenticated) {
      router.push('/login');
    }
  }, [isAuthenticated, loading, router]);

  if (loading || !isAuthenticated) {
    return <div>Loading...</div>; // Or a proper loading spinner
  }

  return (
    <div className="flex flex-col min-h-screen">
      <header className="flex items-center justify-between p-4 border-b">
        <nav className="flex items-center gap-4">
          <Link href="/categories">
            <Button variant="outline">Категории</Button>
          </Link>
          <Link href="/products">
            <Button variant="outline">Товары</Button>
          </Link>
          {user?.role === 'admin' && (
            <Link href="/bot-users">
              <Button variant="outline">Пользователи бота</Button>
            </Link>
          )}
        </nav>
        <Button variant="outline" onClick={logout}>Выход</Button>
      </header>
      <main className="flex-1 p-8">
        {children}
      </main>
    </div>
  );
}
