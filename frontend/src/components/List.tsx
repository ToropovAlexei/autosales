import { Card, CardContent, CardHeader } from "@mui/material";

interface ListProps {
  title: string;
  children: React.ReactNode;
  addButton?: React.ReactNode;
}

export function List({ title, children, addButton }: ListProps) {
  return (
    <Card>
      <CardHeader title={title} action={addButton} />
      <CardContent>{children}</CardContent>
    </Card>
  );
}
