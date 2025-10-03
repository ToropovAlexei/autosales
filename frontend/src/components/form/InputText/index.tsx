import { BaseTextFieldProps, TextField } from "@mui/material";
import { useController } from "react-hook-form";

interface IProps
  extends Omit<BaseTextFieldProps, "name" | "onChange" | "value"> {
  name: string;
  label?: string;
}

export const InputText = ({ name, label, ...props }: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({ name, defaultValue: "" });

  return (
    <TextField
      name={name}
      label={label ?? name}
      error={!!error}
      helperText={error?.message}
      value={value}
      onChange={onChange}
      size="small"
      {...props}
    />
  );
};
