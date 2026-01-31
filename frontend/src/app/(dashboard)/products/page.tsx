"use client";

import { useState, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useDataGrid, useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { Button, Stack } from "@mui/material";
import { ProductForm } from "./components/ProductForm";
import { ProductsTable } from "./components/ProductsTable";
import { ProductCSVUploadModal } from "./components/ProductCSVUploadModal";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { PageLayout } from "@/components/PageLayout";
import { Category, Product } from "@/types";
import { flattenCategoriesForSelect } from "../categories/utils";

export default function ProductsPage() {
  const queryClient = useQueryClient();
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] = useState<Product | null>(null);
  const [isUploadModalOpen, setIsUploadModalOpen] = useState(false);
  const {
    rows: products,
    rowCount,
    loading: isLoadingProducts,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
    sortModel,
    onSortModelChange,
    refetch,
  } = useDataGrid<Product>(ENDPOINTS.PRODUCTS);

  const { data: categories, isPending: isLoadingCategories } =
    useList<Category>({
      endpoint: ENDPOINTS.CATEGORIES,
    });

  const flattenedCategories = useMemo(
    () => (categories?.data ? flattenCategoriesForSelect(categories.data) : []),
    [categories],
  );

  const mutation = useMutation({
    mutationFn: (payload: Partial<Product>) => {
      const params = { url: ENDPOINTS.PRODUCTS, params: payload };
      if (payload.id) {
        return dataLayer.update({ ...params, id: payload.id });
      }
      return dataLayer.create(params);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.PRODUCTS),
      });
      setIsFormOpen(false);
      setSelectedProduct(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) =>
      dataLayer.delete({ url: ENDPOINTS.PRODUCTS, id }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.PRODUCTS),
      });
    },
  });

  const openForm = (product?: Product) => {
    setSelectedProduct(product || null);
    setIsFormOpen(true);
  };

  const handleConfirmForm = (data: Product) => {
    mutation.mutate(data);
  };

  if (isLoadingCategories) return <div>Загрузка...</div>;

  return (
    <PageLayout title="Товары">
      <Stack direction="row" mb={2} gap={2}>
        <Button variant="contained" onClick={() => openForm()}>
          Добавить товар
        </Button>
        <Button variant="outlined" onClick={() => setIsUploadModalOpen(true)}>
          Загрузить CSV
        </Button>
        <Button onClick={() => refetch()}>Обновить</Button>
      </Stack>
      <ProductsTable
        products={products}
        onEdit={openForm}
        onDelete={deleteMutation.mutate}
        loading={isLoadingProducts}
        rowCount={rowCount}
        paginationModel={paginationModel}
        onPaginationModelChange={onPaginationModelChange}
        filterModel={filterModel}
        onFilterModelChange={onFilterModelChange}
        sortModel={sortModel}
        onSortModelChange={onSortModelChange}
        categories={categories?.data || []}
      />
      {isFormOpen && (
        <ProductForm
          open={isFormOpen}
          onClose={() => {
            setIsFormOpen(false);
            setSelectedProduct(null);
          }}
          onConfirm={handleConfirmForm}
          defaultValues={selectedProduct || undefined}
          categories={flattenedCategories}
          allCategories={categories?.data || []}
        />
      )}

      <ProductCSVUploadModal
        open={isUploadModalOpen}
        onClose={() => setIsUploadModalOpen(false)}
      />
    </PageLayout>
  );
}
