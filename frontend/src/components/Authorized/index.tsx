import { ENDPOINTS } from "@/constants";
import { useOne } from "@/hooks";
import { AdminUser } from "@/types";
import { useRouter } from "next/navigation";
import { PropsWithChildren, useEffect } from "react";

export const Authorized = ({ children }: PropsWithChildren) => {
  const { data, isPending } = useOne<AdminUser>({
    endpoint: ENDPOINTS.USERS_ME,
  });
  const router = useRouter();

  useEffect(() => {
    if (!isPending && !data) {
      router.push("/login");
    }
  }, [data, isPending, router]);

  if (isPending) {
    return <div>Загрузка...</div>;
  }

  if (data) {
    return <>{children}</>;
  }

  return null;
};
