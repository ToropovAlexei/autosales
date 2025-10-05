import { BaseTextFieldProps, TextField } from "@mui/material";
import { useController } from "react-hook-form";

interface IProps
  extends Omit<BaseTextFieldProps, "name" | "onChange" | "value"> {
  name: string;
  label?: string;
}

export const InputNumber = ({ name, label, required, ...props }: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({
    name,
    defaultValue: 0,
    rules: {
      required: required && "Поле обязательно к заполнению",
    },
  });

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    onChange(value === "" ? null : parseFloat(value));
  };

  return (
    <TextField
      name={name}
      label={label ?? name}
      type="number"
      error={!!error}
      helperText={error?.message}
      value={value}
      onChange={handleChange}
      size="small"
      required={required}
      {...props}
    />
  );
};
