import { DateTimePicker } from "@mui/x-date-pickers";
import { useController } from "react-hook-form";
import dayjs from "dayjs";

interface IProps {
  name: string;
  label: string;
}

export const InputDateTime = ({ name, label }: IProps) => {
  const {
    field: { onChange, value },
    fieldState: { error },
  } = useController({ name });

  return (
    <DateTimePicker
      label={label}
      name={name}
      value={value ? dayjs(value) : null}
      onChange={(value) =>
        onChange(value && value.isValid() ? value.toISOString() : null)
      }
      slotProps={{
        textField: {
          error: !!error,
          helperText: error?.message,
          size: "small",
        },
      }}
    />
  );
};
