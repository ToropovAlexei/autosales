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
import { api } from "@/lib/api";
import { useRouter } from "next/navigation";
import classes from "./styles.module.css";
import {
  LoginStep1,
  LoginStep1Response,
  LoginStep2,
  LoginStep2Response,
} from "@/types";

type LoginForm = { login: string; password: string; code?: string };

export default function LoginPage() {
  const form = useForm<LoginForm>();
  const { handleSubmit } = form;
  const router = useRouter();

  const {
    mutate: loginMutate,
    error: loginError,
    data: step1Response,
  } = useMutation({
    mutationFn: (form: LoginStep1) =>
      api.post<LoginStep1Response>("auth/login", { json: form }).json(),
  });

  const { mutate: tfaMutate, error: tfaError } = useMutation({
    mutationFn: (form: LoginStep2) =>
      api.post<LoginStep2Response>("auth/login/2fa", { json: form }).json(),
    onSuccess: ({ token }) => {
      localStorage.setItem("jwt", token);
      router.push("/post-login");
    },
  });

  return (
    <main className={classes.main}>
      <FormProvider {...form}>
        <Card>
          <CardHeader
            title={step1Response ? "Двухфакторная аутентификация" : "Вход"}
            subheader={
              step1Response
                ? "Введите код из вашего приложения для аутентификации."
                : "Введите свой логин и пароль для входа в панель."
            }
          />
          <CardContent>
            <Stack gap={1}>
              {!step1Response ? (
                <>
                  <InputText
                    name="login"
                    label="Логин"
                    placeholder="admin"
                    required
                  />
                  <InputPassword name="password" />
                  {loginError && (
                    <Typography color="error">
                      Неверный логин или пароль
                    </Typography>
                  )}
                </>
              ) : (
                <>
                  <InputText
                    name="code"
                    placeholder="123456"
                    required
                    label="Код"
                    focused
                  />
                  {tfaError && (
                    <Typography color="error">Неверный код</Typography>
                  )}
                </>
              )}
              <Button
                onClick={() => {
                  handleSubmit((form) => {
                    if (step1Response) {
                      if (!form.code) {
                        return;
                      }
                      tfaMutate({
                        code: form.code,
                        temp_token: step1Response.temp_token,
                      });
                      return;
                    }
                    loginMutate(form);
                  })();
                }}
              >
                {step1Response ? "Подтвердить" : "Войти"}
              </Button>
            </Stack>
          </CardContent>
        </Card>
      </FormProvider>
    </main>
  );
}
