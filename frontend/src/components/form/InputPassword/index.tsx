import { TextField } from "@mui/material";
import { RegisterOptions, useController } from "react-hook-form";

interface IProps {
  name: string;
  label?: string;
  rules?: RegisterOptions;
}

export const InputPassword = ({ name, label, rules }: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({ name, rules, defaultValue: "" });

  return (
    <TextField
      name={name}
      label={label ?? name}
      error={!!error}
      helperText={error?.message}
      value={value}
      onChange={onChange}
      size="small"
      type="password"
    />
  );
};
