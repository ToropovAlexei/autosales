import {
  FormControl,
  FormControlLabel,
  FormHelperText,
  Switch,
} from "@mui/material";
import { useController } from "react-hook-form";

interface IProps {
  name: string;
  label?: string;
  disabled?: boolean;
  onChangeEffect?: (value: boolean) => void;
}

export const InputSwitch = ({
  name,
  label,
  disabled,
  onChangeEffect,
}: IProps) => {
  const {
    field: { value, ...field },
    fieldState: { error },
  } = useController({ name });

  return (
    <FormControl error={!!error} disabled={disabled}>
      <FormControlLabel
        label={label}
        control={
          <Switch
            {...field}
            onChange={(event) => {
              field.onChange(event);
              onChangeEffect?.(event.target.checked);
            }}
            checked={value ?? false}
            color={error ? "error" : "primary"}
          />
        }
      />
      {error?.message && <FormHelperText>{error?.message}</FormHelperText>}
    </FormControl>
  );
};
