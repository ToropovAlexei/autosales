import {
  FormControl,
  FormHelperText,
  InputLabel,
  MenuItem,
  Select,
} from "@mui/material";
import { useController } from "react-hook-form";

type Option = {
  value: string | number;
  label: string;
};

interface IProps {
  name: string;
  options?: Option[];
  label?: string;
  disabled?: boolean;
}

export const InputSelect = ({ name, label, options }: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({ name, defaultValue: "" });

  return (
    <FormControl error={!!error}>
      <InputLabel id={name}>{label ?? name}</InputLabel>
      <Select
        labelId={name}
        value={value}
        onChange={onChange}
        size="small"
        label={label ?? name}
      >
        {options?.map(({ value, label }) => (
          <MenuItem key={value} value={value}>
            {label}
          </MenuItem>
        ))}
      </Select>
      {error && <FormHelperText error>{error.message}</FormHelperText>}
    </FormControl>
  );
};
