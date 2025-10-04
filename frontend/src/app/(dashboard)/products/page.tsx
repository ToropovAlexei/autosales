"use client";

import { useState, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { ICategory, IProduct } from "@/types";
import { flattenCategoriesForSelect, findCategoryNameById } from "@/lib/utils";
import {
  Button,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  OutlinedInput,
} from "@mui/material";
import { ProductForm } from "./components/ProductForm";
import { ProductsTable } from "./components/ProductsTable";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";

export default function ProductsPage() {
  const queryClient = useQueryClient();
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] = useState<IProduct | null>(null);
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);

  const { data: products, isPending: isLoadingProducts } = useList<IProduct>({
    endpoint: ENDPOINTS.PRODUCTS,
    filter: { "category_ids[]": selectedCategories },
  });

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

  if (isLoadingProducts || isLoadingCategories) return <div>Loading...</div>;

  return (
    <>
      <List
        title="Товары"
        addButton={
          <Button variant="contained" onClick={() => openForm()}>
            Добавить товар
          </Button>
        }
      >
        <FormControl fullWidth margin="normal">
          <InputLabel>Фильтр по категориям</InputLabel>
          <Select
            multiple
            size="small"
            value={selectedCategories}
            onChange={(e) => setSelectedCategories(e.target.value as string[])}
            input={<OutlinedInput label="Фильтр по категориям" />}
          >
            {flattenedCategories.map((cat) => (
              <MenuItem key={cat.id} value={cat.id.toString()}>
                {cat.name}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <ProductsTable
          products={products?.data || []}
          onEdit={openForm}
          onDelete={deleteMutation.mutate}
          getCategoryName={getCategoryName}
        />
      </List>
      {isFormOpen && (
        <ProductForm
          open={isFormOpen}
          onClose={() => setIsFormOpen(false)}
          onConfirm={handleConfirmForm}
          defaultValues={selectedProduct || undefined}
          categories={flattenedCategories.map((c) => ({
            value: c.id,
            label: c.name,
          }))}
        />
      )}
    </>
  );
}
