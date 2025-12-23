import {
  FormControl,
  FormHelperText,
  InputLabel,
  MenuItem,
  Select,
} from "@mui/material";
import { RegisterOptions, useController } from "react-hook-form";

type Option = {
  value: string | number;
  label: string;
};

interface IProps {
  name: string;
  options?: Option[];
  label?: string;
  disabled?: boolean;
  rules?: RegisterOptions;
  withNone?: boolean;
}

export const InputSelect = ({
  name,
  label,
  options,
  rules,
  withNone,
}: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({ name, rules, defaultValue: "" });

  return (
    <FormControl error={!!error}>
      <InputLabel id={name} size="small">
        {label ?? name}
      </InputLabel>
      <Select
        labelId={name}
        value={value}
        onChange={onChange}
        size="small"
        label={label ?? name}
      >
        {withNone && <MenuItem value="">Не выбрано</MenuItem>}
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
