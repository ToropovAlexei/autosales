
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface ListProps {
  title: string;
  children: React.ReactNode;
  addButton?: React.ReactNode;
}

export function List({ title, children, addButton }: ListProps) {
  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>{title}</CardTitle>
          {addButton}
        </div>
      </CardHeader>
      <CardContent>
        {children}
      </CardContent>
    </Card>
  );
}
