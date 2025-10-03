import { ENDPOINTS } from "@/constants";
import { useOne } from "@/hooks";
import { User } from "@/types/common";
import { useRouter } from "next/navigation";
import { PropsWithChildren, useEffect } from "react";

export const Authorized = ({ children }: PropsWithChildren) => {
  const { data, isPending } = useOne<User>({ endpoint: ENDPOINTS.USERS_ME });
  const router = useRouter();

  useEffect(() => {
    if (!isPending && !data) {
      router.push("/login");
    }
  }, [data, isPending, router]);

  if (isPending) {
    return <div>Loading...</div>;
  }

  if (data) {
    return <>{children}</>;
  }

  return null;
};
