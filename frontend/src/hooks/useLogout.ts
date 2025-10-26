import { useQueryClient } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { useCallback } from "react";
import { newApi } from "../lib/api";

export const useLogout = () => {
  const client = useQueryClient();
  const router = useRouter();

  const logout = useCallback(() => {
    newApi.post('auth/logout').then(() => {
      localStorage.removeItem("jwt");
      router.push("/login");
      client.clear();
    });
  }, [client, router]);

  return logout;
};
