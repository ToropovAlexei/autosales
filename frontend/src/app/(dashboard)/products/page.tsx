"use client";

import { useState, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useDataGrid, useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { ICategory, IProduct } from "@/types";
import { flattenCategoriesForSelect, findCategoryNameById } from "@/lib/utils";
import { Button } from "@mui/material";
import { ProductForm } from "./components/ProductForm";
import { ProductsTable } from "./components/ProductsTable";
import { ProductCSVUploadModal } from "./components/ProductCSVUploadModal";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { PageLayout } from "@/components/PageLayout";

export default function ProductsPage() {
  const queryClient = useQueryClient();
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] = useState<IProduct | null>(null);
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
  } = useDataGrid(ENDPOINTS.PRODUCTS);

  const { data: categories, isPending: isLoadingCategories } =
    useList<ICategory>({
      endpoint: ENDPOINTS.CATEGORIES,
    });

  const flattenedCategories = useMemo(
    () => (categories?.data ? flattenCategoriesForSelect(categories.data) : []),
    [categories]
  );

  const getCategoryName = (categoryId: number) => {
    if (!categoryId) return "N/A";
    return findCategoryNameById(categories?.data || [], categoryId) || "N/A";
  };

  const mutation = useMutation({
    mutationFn: (payload: Partial<IProduct>) => {
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

  const openForm = (product?: IProduct) => {
    setSelectedProduct(product || null);
    setIsFormOpen(true);
  };

  const handleConfirmForm = (data: Partial<IProduct>) => {
    mutation.mutate(data);
  };

  if (isLoadingCategories) return <div>Загрузка...</div>;

  return (
    <PageLayout title="Товары">
      <Button variant="contained" onClick={() => openForm()} sx={{ mb: 2, mr: 2 }}>
        Добавить товар
      </Button>
      <Button variant="outlined" onClick={() => setIsUploadModalOpen(true)} sx={{ mb: 2 }}>
        Загрузить CSV
      </Button>
      <ProductsTable
        products={products}
        onEdit={openForm}
        onDelete={deleteMutation.mutate}
        getCategoryName={getCategoryName}
        loading={isLoadingProducts}
        rowCount={rowCount}
        paginationModel={paginationModel}
        onPaginationModelChange={onPaginationModelChange}
        filterModel={filterModel}
        onFilterModelChange={onFilterModelChange}
        sortModel={sortModel}
        onSortModelChange={onSortModelChange}
        categories={flattenedCategories}
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
          categories={flattenedCategories.map((c) => ({
            value: c.id,
            label: c.name,
          }))}
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
