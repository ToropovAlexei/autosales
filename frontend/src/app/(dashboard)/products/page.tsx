"use client";

import { useState, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import api from "@/lib/api";
import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { MultiSelect } from "@/components/ui/multi-select";
import { CategoryResponse } from "@/types";
import { flattenCategoriesForSelect, findCategoryNameById } from "@/lib/utils";

interface Product {
  id: number;
  name: string;
  category_id: number;
  price: number;
  stock: number;
  type: "item" | "subscription";
  subscription_period_days: number;
  provider?: string;
  external_id?: string;
}

interface ProductFormData {
  name: string;
  category_id: number;
  price: number;
  initial_stock: number;
  type: "item" | "subscription";
  subscription_period_days: number;
}

export default function ProductsPage() {
  const queryClient = useQueryClient();
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] = useState<Product | null>(null);
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);

  // Form state
  const [name, setName] = useState("");
  const [categoryId, setCategoryId] = useState<number | null>(null);
  const [price, setPrice] = useState("");
  const [initialStock, setInitialStock] = useState("");
  const [productType, setProductType] = useState<"item" | "subscription">(
    "item"
  );
  const [subscriptionDays, setSubscriptionDays] = useState("30");

  const { data: products, isLoading: isLoadingProducts } = useList<Product>({
    endpoint: ENDPOINTS.PRODUCTS,
    filter: { "category_ids[]": selectedCategories },
  });

  const { data: categories, isLoading: isLoadingCategories } =
    useList<CategoryResponse>({ endpoint: ENDPOINTS.CATEGORIES });

  const flattenedCategories = useMemo(
    () => (categories?.data ? flattenCategoriesForSelect(categories.data) : []),
    [categories]
  );

  const getCategoryName = (categoryId: number) => {
    if (!categoryId) return "N/A";
    return findCategoryNameById(categories?.data || [], categoryId) || "N/A";
  };

  const addMutation = useMutation({
    mutationFn: (newProduct: Partial<ProductFormData>) =>
      api.post("/products", newProduct),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["products"] });
      // Reset form
      setName("");
      setCategoryId(null);
      setPrice("");
      setInitialStock("");
      setProductType("item");
      setSubscriptionDays("30");
      setIsAddOpen(false);
    },
  });

  const editMutation = useMutation({
    mutationFn: (updatedProduct: Omit<Product, "stock">) =>
      api.put(`/products/${updatedProduct.id}`, {
        name: updatedProduct.name,
        category_id: updatedProduct.category_id,
        price: updatedProduct.price,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["products"] });
      setSelectedProduct(null);
      setIsEditOpen(false);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.delete(`/products/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["products"] });
    },
  });

  const handleAddProduct = () => {
    if (name.trim() === "" || !categoryId || !price) return;

    const newProduct: Partial<ProductFormData> = {
      name,
      category_id: categoryId,
      price: parseFloat(price),
      type: productType,
    };

    if (productType === "item") {
      newProduct.initial_stock = parseInt(initialStock, 10) || 0;
      newProduct.subscription_period_days = 0;
    } else {
      newProduct.initial_stock = 0;
      newProduct.subscription_period_days =
        parseInt(subscriptionDays, 10) || 30;
    }

    addMutation.mutate(newProduct);
  };

  const handleEditProduct = () => {
    if (selectedProduct) {
      const { stock, ...productToUpdate } = selectedProduct;
      editMutation.mutate(productToUpdate);
    }
  };

  const openEditDialog = (product: Product) => {
    setSelectedProduct(product);
    setIsEditOpen(true);
  };

  if (isLoadingProducts || isLoadingCategories) return <div>Loading...</div>;

  return (
    <>
      <List
        title="Товары"
        addButton={
          <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
            <DialogTrigger asChild>
              <Button>Добавить товар</Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
              <DialogHeader>
                <DialogTitle>Добавить товар</DialogTitle>
                <DialogDescription>
                  Заполните информацию о новом товаре.
                </DialogDescription>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid items-center grid-cols-4 gap-4">
                  <Label htmlFor="type" className="text-right">
                    Тип
                  </Label>
                  <Select
                    onValueChange={(value: "item" | "subscription") =>
                      setProductType(value)
                    }
                    defaultValue="item"
                  >
                    <SelectTrigger className="col-span-3">
                      <SelectValue placeholder="Выберите тип" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="item">Товар</SelectItem>
                      <SelectItem value="subscription">Подписка</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid items-center grid-cols-4 gap-4">
                  <Label htmlFor="name" className="text-right">
                    Название
                  </Label>
                  <Input
                    id="name"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="col-span-3"
                  />
                </div>
                <div className="grid items-center grid-cols-4 gap-4">
                  <Label htmlFor="category" className="text-right">
                    Категория
                  </Label>
                  <Select
                    onValueChange={(value) => setCategoryId(Number(value))}
                  >
                    <SelectTrigger className="col-span-3">
                      <SelectValue placeholder="Выберите категорию" />
                    </SelectTrigger>
                    <SelectContent>
                      {flattenedCategories.map((cat) => (
                        <SelectItem key={cat.id} value={cat.id.toString()}>
                          {cat.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid items-center grid-cols-4 gap-4">
                  <Label htmlFor="price" className="text-right">
                    Цена
                  </Label>
                  <Input
                    id="price"
                    type="number"
                    value={price}
                    onChange={(e) => setPrice(e.target.value)}
                    className="col-span-3"
                  />
                </div>
                {productType === "item" ? (
                  <div className="grid items-center grid-cols-4 gap-4">
                    <Label htmlFor="initial_stock" className="text-right">
                      Начальный остаток
                    </Label>
                    <Input
                      id="initial_stock"
                      type="number"
                      value={initialStock}
                      onChange={(e) => setInitialStock(e.target.value)}
                      className="col-span-3"
                    />
                  </div>
                ) : (
                  <div className="grid items-center grid-cols-4 gap-4">
                    <Label htmlFor="subscription_days" className="text-right">
                      Срок (дней)
                    </Label>
                    <Input
                      id="subscription_days"
                      type="number"
                      value={subscriptionDays}
                      onChange={(e) => setSubscriptionDays(e.target.value)}
                      className="col-span-3"
                    />
                  </div>
                )}
              </div>
              <DialogFooter>
                <Button
                  type="submit"
                  onClick={handleAddProduct}
                  disabled={addMutation.isPending}
                >
                  {addMutation.isPending ? "Добавление..." : "Добавить"}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        }
      >
        <div className="mb-4">
          <MultiSelect
            options={flattenedCategories.map((cat) => ({
              value: cat.id.toString(),
              label: cat.name,
            }))}
            selected={selectedCategories}
            onChange={setSelectedCategories}
            placeholder="Фильтр по категориям"
          />
        </div>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>ID</TableHead>
              <TableHead>Название</TableHead>
              <TableHead>Тип</TableHead>
              <TableHead>Категория</TableHead>
              <TableHead>Цена</TableHead>
              <TableHead>Остаток</TableHead>
              <TableHead className="text-right">Действия</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {products?.data?.map((product) => (
              <TableRow key={product.id || product.external_id}>
                <TableCell>{product.provider ? "-" : product.id}</TableCell>
                <TableCell>{product.name}</TableCell>
                <TableCell>
                  {product.provider
                    ? `Внешний (${product.provider})`
                    : product.type === "subscription"
                    ? `Подписка (${product.subscription_period_days} дн.)`
                    : "Товар"}
                </TableCell>
                <TableCell>{getCategoryName(product.category_id)}</TableCell>
                <TableCell>{product.price} ₽</TableCell>
                <TableCell>
                  {product.type === "subscription" ? "∞" : product.stock}
                </TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => openEditDialog(product)}
                    disabled={!!product.provider}
                  >
                    ✏️
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => deleteMutation.mutate(product.id)}
                    disabled={!!product.provider}
                  >
                    🗑️
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </List>

      {/* Edit Dialog */}
      {selectedProduct && (
        <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
          <DialogContent className="sm:max-w-[425px]">
            <DialogHeader>
              <DialogTitle>Редактировать товар</DialogTitle>
              <DialogDescription>
                Обновите информацию о товаре.
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid items-center grid-cols-4 gap-4">
                <Label htmlFor="edit-name" className="text-right">
                  Название
                </Label>
                <Input
                  id="edit-name"
                  value={selectedProduct.name}
                  onChange={(e) =>
                    setSelectedProduct((p) =>
                      p ? { ...p, name: e.target.value } : null
                    )
                  }
                  className="col-span-3"
                />
              </div>
              <div className="grid items-center grid-cols-4 gap-4">
                <Label htmlFor="edit-category" className="text-right">
                  Категория
                </Label>
                <Select
                  onValueChange={(value) =>
                    setSelectedProduct((p) =>
                      p ? { ...p, category_id: Number(value) } : null
                    )
                  }
                  value={selectedProduct.category_id.toString()}
                >
                  <SelectTrigger className="col-span-3">
                    <SelectValue placeholder="Выберите категорию" />
                  </SelectTrigger>
                  <SelectContent>
                    {flattenedCategories.map((cat) => (
                      <SelectItem key={cat.id} value={cat.id.toString()}>
                        {cat.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid items-center grid-cols-4 gap-4">
                <Label htmlFor="edit-price" className="text-right">
                  Цена
                </Label>
                <Input
                  id="edit-price"
                  type="number"
                  value={selectedProduct.price}
                  onChange={(e) =>
                    setSelectedProduct((p) =>
                      p ? { ...p, price: Number(e.target.value) } : null
                    )
                  }
                  className="col-span-3"
                />
              </div>
            </div>
            <DialogFooter>
              <Button
                type="submit"
                onClick={handleEditProduct}
                disabled={editMutation.isPending}
              >
                {editMutation.isPending ? "Сохранение..." : "Сохранить"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}
    </>
  );
}
