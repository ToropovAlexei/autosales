import { BaseTextFieldProps, TextField } from "@mui/material";
import { RegisterOptions, useController } from "react-hook-form";

interface IProps
  extends Omit<BaseTextFieldProps, "name" | "onChange" | "value"> {
  name: string;
  label?: string;
  rules?: RegisterOptions;
}

export const InputNumber = ({
  name,
  label,
  required,
  rules,
  ...props
}: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({
    name,
    defaultValue: null,
    rules: {
      required: required && "Поле обязательно к заполнению",
      ...rules,
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
      value={value || ""}
      onChange={handleChange}
      size="small"
      required={required}
      {...props}
    />
  );
};
