"use client";

import { FormProvider, useForm } from "react-hook-form";
import {
  Button,
  Card,
  CardContent,
  CardHeader,
  Stack,
  Typography,
} from "@mui/material";
import { InputPassword, InputText } from "@/components";
import { useMutation } from "@tanstack/react-query";
import { newApi } from "@/lib/api";
import { useRouter } from "next/navigation";
import classes from "./styles.module.css";

type FormData = { email: string; password: string };

export default function LoginPage() {
  const form = useForm<FormData>();
  const { handleSubmit } = form;
  const router = useRouter();

  const { mutate, error: loginError } = useMutation({
    mutationFn: (form: FormData) =>
      newApi
        .post<{ data: { access_token: string } }>("auth/login", { json: form })
        .json()
        .then(({ data }) => data.access_token),
    onSuccess: (token) => {
      localStorage.setItem("jwt", token);
      router.push("/dashboard");
    },
  });

  return (
    <main className={classes.main}>
      <FormProvider {...form}>
        <Card component="form" onSubmit={handleSubmit((form) => mutate(form))}>
          <CardHeader
            title="Вход"
            subheader="Введите свой email и пароль для входа в панель."
          />
          <CardContent>
            <Stack gap={1}>
              <InputText
                name="email"
                type="email"
                placeholder="m@example.com"
                required
              />
              <InputPassword name="password" />
              {loginError && (
                <Typography color="error">Неверный email или пароль</Typography>
              )}
              <Button type="submit">Войти</Button>
            </Stack>
          </CardContent>
        </Card>
      </FormProvider>
    </main>
  );
}
