import { BaseTextFieldProps, TextField } from "@mui/material";
import { useController } from "react-hook-form";

interface IProps
  extends Omit<BaseTextFieldProps, "name" | "onChange" | "value"> {
  name: string;
  label?: string;
}

export const InputText = ({ name, label, required, ...props }: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({
    name,
    defaultValue: "",
    rules: {
      required: required && "Поле обязательно к заполнению",
    },
  });

  return (
    <TextField
      name={name}
      label={label ?? name}
      error={!!error}
      helperText={error?.message}
      value={value}
      onChange={onChange}
      size="small"
      required={required}
      {...props}
    />
  );
};
