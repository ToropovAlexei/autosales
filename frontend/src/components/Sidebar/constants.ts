import HomeRoundedIcon from "@mui/icons-material/HomeRounded";
import CategoryIcon from "@mui/icons-material/Category";
import ShoppingCartIcon from "@mui/icons-material/ShoppingCart";
import GroupIcon from "@mui/icons-material/Group";
import ReceiptIcon from "@mui/icons-material/Receipt";
import InventoryIcon from "@mui/icons-material/Inventory";
import SmartToyIcon from "@mui/icons-material/SmartToy";
import LocalMallIcon from "@mui/icons-material/LocalMall";

export const MENU_ITEMS = [
  {
    label: "Дашборд",
    Icon: HomeRoundedIcon,
    path: "/dashboard",
  },
  {
    label: "Категории",
    Icon: CategoryIcon,
    path: "/categories",
  },
  {
    label: "Товары",
    Icon: ShoppingCartIcon,
    path: "/products",
  },
  {
    label: "Пользователи бота",
    Icon: GroupIcon,
    path: "/bot-users",
  },
  {
    label: "Транзакции",
    Icon: ReceiptIcon,
    path: "/transactions",
  },
  {
    label: "Заказы",
    Icon: LocalMallIcon,
    path: "/orders",
  },
  {
    label: "Склад",
    Icon: InventoryIcon,
    path: "/stock",
  },
  {
    label: "Реферальные боты",
    Icon: SmartToyIcon,
    path: "/referral-bots",
  },
];
