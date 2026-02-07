"use client";

import { PageLayout } from "@/components/PageLayout";
import { ImageFoldersManager } from "@/components/ImageFoldersManager";

export default function ImagesPage() {
  return (
    <PageLayout title="Управление изображениями">
      <ImageFoldersManager />
    </PageLayout>
  );
}
