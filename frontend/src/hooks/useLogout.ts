import { useQueryClient } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { useCallback } from "react";
import { api } from "../lib/api";

export const useLogout = () => {
  const client = useQueryClient();
  const router = useRouter();

  const logout = useCallback(() => {
    api.post("auth/logout").then(() => {
      localStorage.removeItem("jwt");
      router.push("/login");
      client.clear();
    });
  }, [client, router]);

  return logout;
};
