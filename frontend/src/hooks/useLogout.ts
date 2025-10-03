import { useQueryClient } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { useCallback } from "react";

export const useLogout = () => {
  const client = useQueryClient();
  const router = useRouter();

  const logout = useCallback(() => {
    localStorage.removeItem("jwt");
    router.push("/login");
    client.clear();
  }, [client, router]);

  return logout;
};
