import { ROUTE_BY_PATHNAME, ROUTES_ACCESS_MAP } from "@/constants";
import { useCan } from "@/hooks";
import { usePathname, useRouter } from "next/navigation";
import { PropsWithChildren, useEffect } from "react";

export const PageAccessGuard = ({ children }: PropsWithChildren) => {
  const pathname = usePathname();
  const { can, isLoading } = useCan(
    ROUTES_ACCESS_MAP[ROUTE_BY_PATHNAME[pathname]]
  );
  const router = useRouter();

  useEffect(() => {
    if (!can && !isLoading) {
      router.push("/");
    }
  }, [can, isLoading, pathname]);

  if (!can) {
    return null;
  }

  return <>{children}</>;
};
