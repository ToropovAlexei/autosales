"use client";

import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
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

interface Product {
  id: number;
  name: string;
  category_id: number;
  price: number;
  stock: number;
}

interface Category {
  id: number;
  name: string;
}

interface ProductFormData {
  name: string;
  category_id: number;
  price: number;
  initial_stock: number;
}

export default function ProductsPage() {
  const queryClient = useQueryClient();
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] = useState<Product | null>(null);

  // Form state
  const [name, setName] = useState("");
  const [categoryId, setCategoryId] = useState<number | null>(null);
  const [price, setPrice] = useState("");
  const [initialStock, setInitialStock] = useState("");

  const { data: products, isLoading: isLoadingProducts } = useQuery<Product[]>({
    queryKey: ["products"],
    queryFn: () => api.get("/products"),
  });

  const { data: categories, isLoading: isLoadingCategories } = useQuery<
    Category[]
  >({
    queryKey: ["categories"],
    queryFn: () => api.get("/categories"),
  });

  const getCategoryName = (categoryId: number) => {
    return categories?.find((cat) => cat.id === categoryId)?.name || "N/A";
  };

  const addMutation = useMutation({
    mutationFn: (newProduct: ProductFormData) =>
      api.post("/products", newProduct),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["products"] });
      setName("");
      setCategoryId(null);
      setPrice("");
      setInitialStock("");
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
    if (name.trim() !== "" && categoryId && price && initialStock) {
      addMutation.mutate({
        name,
        category_id: categoryId,
        price: parseFloat(price),
        initial_stock: parseInt(initialStock, 10),
      });
    }
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
                      {categories?.map((cat) => (
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
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>ID</TableHead>
              <TableHead>Название</TableHead>
              <TableHead>Категория</TableHead>
              <TableHead>Цена</TableHead>
              <TableHead>Остаток</TableHead>
              <TableHead className="text-right">Действия</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {products?.map((product) => (
              <TableRow key={product.id}>
                <TableCell>{product.id}</TableCell>
                <TableCell>{product.name}</TableCell>
                <TableCell>{getCategoryName(product.category_id)}</TableCell>
                <TableCell>{product.price} ₽</TableCell>
                <TableCell>{product.stock}</TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => openEditDialog(product)}
                  >
                    ✏️
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => deleteMutation.mutate(product.id)}
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
                    {categories?.map((cat) => (
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
