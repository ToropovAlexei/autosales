import { Autocomplete, AutocompleteProps, TextField } from "@mui/material";
import { RegisterOptions, useController } from "react-hook-form";

type MuiAutocompleteProps = AutocompleteProps<
  string,
  boolean,
  boolean,
  boolean
>;

interface IProps extends Omit<
  MuiAutocompleteProps,
  "renderInput" | "value" | "onChange" | "multiple" | "freeSolo" | "options"
> {
  name: string;
  label?: string;
  rules?: RegisterOptions;
  options: string[];
  multiple?: boolean;
  freeSolo?: boolean;
  placeholder?: string;
  required?: boolean;
}

export const InputAutocomplete = ({
  name,
  label,
  required,
  rules,
  options,
  multiple = false,
  freeSolo = false,
  placeholder,
  ...props
}: IProps) => {
  const {
    field: { value, onChange },
    fieldState: { error },
  } = useController({
    name,
    defaultValue: multiple ? [] : "",
    rules: {
      ...rules,
      required: required && "Поле обязательно к заполнению",
    },
  });

  return (
    <Autocomplete
      multiple={multiple}
      freeSolo={freeSolo}
      options={options}
      filterSelectedOptions={multiple}
      value={value ?? (multiple ? [] : "")}
      onChange={(_, newValue) => onChange(newValue)}
      renderInput={(params) => (
        <TextField
          {...params}
          name={name}
          label={label ?? name}
          error={!!error}
          helperText={error?.message}
          required={required}
          size="small"
          placeholder={placeholder}
        />
      )}
      {...props}
    />
  );
};
