"use client";

import { useState } from "react";
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

type LoginForm = { email: string; password: string; code?: string };

export default function LoginPage() {
  const [tfaRequired, setTfaRequired] = useState(false);
  const [tempToken, setTempToken] = useState("");
  const form = useForm<LoginForm>();
  const { handleSubmit } = form;
  const router = useRouter();

  const { mutate: loginMutate, error: loginError } = useMutation({
    mutationFn: (form: LoginForm) =>
      newApi
        .post<{
          data: {
            access_token: string;
            tfa_required?: boolean;
            temp_token?: string;
          };
        }>("auth/login", { json: form })
        .json(),
    onSuccess: (response) => {
      if (response.data.tfa_required) {
        setTfaRequired(true);
        setTempToken(response.data.temp_token || "");
      } else {
        localStorage.setItem("jwt", response.data.access_token);
        router.push("/post-login");
      }
    },
  });

  const { mutate: tfaMutate, error: tfaError } = useMutation({
    mutationFn: (form: LoginForm) =>
      newApi
        .post<{ data: { access_token: string } }>("auth/verify-2fa", {
          json: { temp_token: tempToken, code: form.code },
        })
        .json()
        .then(({ data }) => data.access_token),
    onSuccess: (token) => {
      localStorage.setItem("jwt", token);
      router.push("/post-login");
    },
  });

  return (
    <main className={classes.main}>
      <FormProvider {...form}>
        <Card
          component="form"
          onSubmit={handleSubmit((form) =>
            tfaRequired ? tfaMutate(form) : loginMutate(form)
          )}
        >
          <CardHeader
            title={tfaRequired ? "Two-Factor Authentication" : "Вход"}
            subheader={
              tfaRequired
                ? "Введите код из вашего приложения для аутентификации."
                : "Введите свой email и пароль для входа в панель."
            }
          />
          <CardContent>
            <Stack gap={1}>
              {!tfaRequired ? (
                <>
                  <InputText
                    name="email"
                    type="email"
                    placeholder="m@example.com"
                    required
                  />
                  <InputPassword name="password" />
                  {loginError && (
                    <Typography color="error">
                      Неверный email или пароль
                    </Typography>
                  )}
                  <Button type="submit">Войти</Button>
                </>
              ) : (
                <>
                  <InputText name="code" placeholder="123456" required />
                  {tfaError && (
                    <Typography color="error">Неверный код</Typography>
                  )}
                  <Button type="submit">Подтвердить</Button>
                </>
              )}
            </Stack>
          </CardContent>
        </Card>
      </FormProvider>
    </main>
  );
}
