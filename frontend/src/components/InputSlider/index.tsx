"use client";

import {
  Slider,
  Stack,
  Typography,
  FormControl,
  FormHelperText,
} from "@mui/material";
import { useController } from "react-hook-form";

interface InputSliderProps {
  name: string;
  label: string;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
}

export const InputSlider = ({
  name,
  label,
  min = 0,
  max = 100,
  step = 1,
  disabled = false,
}: InputSliderProps) => {
  const {
    field,
    fieldState: { error },
  } = useController({ name, disabled, defaultValue: min });

  return (
    <FormControl error={!!error} fullWidth>
      <Typography gutterBottom>{label}</Typography>
      <Stack direction="row" spacing={2} alignItems="center">
        <Slider
          {...field}
          value={field.value || min}
          onChange={(_, value) => field.onChange(value)}
          aria-labelledby="input-slider"
          valueLabelDisplay="auto"
          step={step}
          marks
          min={min}
          max={max}
          disabled={disabled}
        />
        <Typography sx={{ minWidth: "40px" }}>{field.value}%</Typography>
      </Stack>
      {error && <FormHelperText>{error.message}</FormHelperText>}
    </FormControl>
  );
};
