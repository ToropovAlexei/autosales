"use client";

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { BotCard } from "./components/BotCard";
import classes from "./styles.module.css";
import { PageLayout } from "@/components/PageLayout";
import { Button } from "@mui/material";
import { BotFormModal } from "./components/BotFormModal";
import { Bot, NewBot, UpdateBot } from "@/types";
import { toast } from "react-toastify";

export default function BotsPage() {
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { data: bots } = useList<Bot>({
    endpoint: ENDPOINTS.BOTS,
    filter: { order_by: "id" },
  });

  const { mutate: createBot, isPending: isCreatePending } = useMutation<
    Bot,
    unknown,
    NewBot
  >({
    mutationFn: (params) =>
      dataLayer.create({
        url: ENDPOINTS.BOTS,
        params,
      }),
    onSuccess: (data) => {
      toast.success(`Бот ${data.username} создан`);
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.BOTS),
      });
      setIsModalOpen(false);
    },
    onError: () => {
      toast.error("Произошла ошибка при создании бота");
    },
  });

  const { mutate: updateBot, isPending: isUpdatePending } = useMutation({
    mutationFn: ({ id, params }: { id: Bot["id"]; params: UpdateBot }) =>
      dataLayer.update({ url: ENDPOINTS.BOTS, id, params }),
    onSuccess: () => {
      toast.success("Настройки бота сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.BOTS),
      });
    },
    onError: () => {
      toast.error("Произошла ошибка при сохранении бота");
    },
  });

  const { mutate: deleteBot, isPending: isDeletePending } = useMutation({
    mutationFn: (botId: Bot["id"]) =>
      dataLayer.delete({ url: ENDPOINTS.BOTS, id: botId }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.BOTS),
      });
    },
  });

  return (
    <PageLayout title="Управление ботами">
      <Button
        variant="contained"
        sx={{ mb: 2 }}
        onClick={() => setIsModalOpen(true)}
      >
        Добавить бота
      </Button>
      <div className={classes.grid}>
        {bots?.data?.map((bot: Bot) => (
          <BotCard
            key={bot.id}
            bot={bot}
            onUpdate={updateBot}
            onDelete={deleteBot}
            isPending={isUpdatePending || isDeletePending}
          />
        ))}
      </div>
      <BotFormModal
        open={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onConfirm={createBot}
        isCreating={isCreatePending}
      />
    </PageLayout>
  );
}
