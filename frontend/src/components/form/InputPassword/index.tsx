import { TextField } from "@mui/material";
import { RegisterOptions, useController } from "react-hook-form";

interface IProps {
  name: string;
  label?: string;
  rules?: RegisterOptions;
  helperText?: string;
}

export const InputPassword = ({ name, label, rules, helperText }: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({ name, rules, defaultValue: "" });

  return (
    <TextField
      name={name}
      label={label ?? name}
      error={!!error}
      helperText={error?.message || helperText}
      value={value}
      onChange={onChange}
      size="small"
      type="password"
    />
  );
};
