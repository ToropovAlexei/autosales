import { DatePicker } from "@mui/x-date-pickers";
import { useController } from "react-hook-form";
import dayjs from "dayjs";

interface IProps {
  name: string;
}

export const InputDate = ({ name }: IProps) => {
  const {
    field: { onChange, value },
    fieldState: { error },
  } = useController({ name });

  return (
    <DatePicker
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
