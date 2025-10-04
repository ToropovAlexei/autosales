'use client';

import { useForm, Controller } from 'react-hook-form';
import { Dialog, DialogTitle, DialogContent, DialogActions, TextField, Button, Select, MenuItem, FormControl, InputLabel } from '@mui/material';
import { ICategory } from '@/types';

interface ProductFormData {
  name: string;
  category_id: number;
  price: number;
  initial_stock: number;
  type: 'item' | 'subscription';
  subscription_period_days: number;
}

interface ProductFormProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (data: ProductFormData) => void;
  defaultValues?: Partial<ProductFormData>;
  categories: { value: number; label: string }[];
}

export const ProductForm = ({ open, onClose, onConfirm, defaultValues, categories }: ProductFormProps) => {
  const { handleSubmit, control, watch, reset } = useForm<ProductFormData>({
    defaultValues: defaultValues || {
      name: '',
      category_id: 0,
      price: 0,
      initial_stock: 0,
      type: 'item',
      subscription_period_days: 30,
    },
  });

  const productType = watch('type');

  const onSubmit = (data: ProductFormData) => {
    onConfirm(data);
    reset();
  };

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm" disableScrollLock>
      <DialogTitle>{defaultValues?.name ? 'Редактировать товар' : 'Добавить товар'}</DialogTitle>
      <form onSubmit={handleSubmit(onSubmit)}>
        <DialogContent>
          <Controller
            name="type"
            control={control}
            render={({ field }) => (
              <FormControl fullWidth margin="normal">
                <InputLabel>Тип</InputLabel>
                <Select {...field}>
                  <MenuItem value="item">Товар</MenuItem>
                  <MenuItem value="subscription">Подписка</MenuItem>
                </Select>
              </FormControl>
            )}
          />
          <Controller
            name="name"
            control={control}
            render={({ field }) => <TextField {...field} label="Название" fullWidth margin="normal" />}
          />
          <Controller
            name="category_id"
            control={control}
            render={({ field }) => (
              <FormControl fullWidth margin="normal">
                <InputLabel>Категория</InputLabel>
                <Select {...field}>
                  {categories.map((cat) => (
                    <MenuItem key={cat.value} value={cat.value}>
                      {cat.label}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            )}
          />
          <Controller
            name="price"
            control={control}
            render={({ field }) => <TextField {...field} label="Цена" type="number" fullWidth margin="normal" />}
          />
          {productType === 'item' ? (
            <Controller
              name="initial_stock"
              control={control}
              render={({ field }) => <TextField {...field} label="Начальный остаток" type="number" fullWidth margin="normal" />}
            />
          ) : (
            <Controller
              name="subscription_period_days"
              control={control}
              render={({ field }) => <TextField {...field} label="Срок (дней)" type="number" fullWidth margin="normal" />}
            />
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={onClose}>Отмена</Button>
          <Button type="submit" variant="contained">
            {defaultValues?.name ? 'Сохранить' : 'Добавить'}
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  );
};
