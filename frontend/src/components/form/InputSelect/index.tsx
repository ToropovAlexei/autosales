import {
  FormControl,
  FormHelperText,
  InputLabel,
  MenuItem,
  Select,
} from "@mui/material";
import { RegisterOptions, useController } from "react-hook-form";

type Option = {
  value: string | number | bigint;
  label: string;
};

interface IProps {
  name: string;
  options?: Option[];
  label?: string;
  disabled?: boolean;
  rules?: RegisterOptions;
  withNone?: boolean;
  noneLabel?: string;
  displayEmpty?: boolean;
}

export const InputSelect = ({
  name,
  label,
  options,
  rules,
  withNone,
  noneLabel,
  displayEmpty,
}: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({ name, rules, defaultValue: null });

  return (
    <FormControl error={!!error}>
      <InputLabel id={name} size="small" shrink={displayEmpty}>
        {label ?? name}
      </InputLabel>
      <Select
        labelId={name}
        displayEmpty={displayEmpty}
        value={withNone ? value || "" : value}
        onChange={onChange}
        size="small"
        label={label ?? name}
      >
        {withNone && <MenuItem value="">{noneLabel || "Не выбрано"}</MenuItem>}
        {options?.map(({ value, label }) => (
          <MenuItem key={value} value={String(value)}>
            {label}
          </MenuItem>
        ))}
      </Select>
      {error && <FormHelperText error>{error.message}</FormHelperText>}
    </FormControl>
  );
};
