import { TextField } from "@mui/material";
import { useController } from "react-hook-form";

interface IProps {
  name: string;
  label?: string;
}

export const InputPassword = ({ name, label }: IProps) => {
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
      type="password"
    />
  );
};
