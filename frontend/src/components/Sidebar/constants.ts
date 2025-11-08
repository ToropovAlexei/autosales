import HomeRoundedIcon from "@mui/icons-material/HomeRounded";
import CategoryIcon from "@mui/icons-material/Category";
import ShoppingCartIcon from "@mui/icons-material/ShoppingCart";
import GroupIcon from "@mui/icons-material/Group";
import ReceiptIcon from "@mui/icons-material/Receipt";
import InventoryIcon from "@mui/icons-material/Inventory";
import SmartToyIcon from "@mui/icons-material/SmartToy";
import LocalMallIcon from "@mui/icons-material/LocalMall";
import SettingsIcon from "@mui/icons-material/Settings";
import PhotoLibraryIcon from "@mui/icons-material/PhotoLibrary";
import AdminPanelSettingsIcon from "@mui/icons-material/AdminPanelSettings";
import HistoryIcon from "@mui/icons-material/History";

export const MENU_ITEMS = [
  {
    label: "Дашборд",
    Icon: HomeRoundedIcon,
    path: "/dashboard",
    permission: "dashboard:read",
  },
  {
    label: "Категории",
    Icon: CategoryIcon,
    path: "/categories",
    permission: "categories:read",
  },
  {
    label: "Товары",
    Icon: ShoppingCartIcon,
    path: "/products",
    permission: "products:read",
  },
  {
    label: "Пользователи бота",
    Icon: GroupIcon,
    path: "/bot-users",
    permission: "users:read", // Assuming bot users fall under general user read
  },
  {
    label: "Транзакции",
    Icon: ReceiptIcon,
    path: "/transactions",
    permission: "transactions:read",
  },
  {
    label: "Заказы",
    Icon: LocalMallIcon,
    path: "/orders",
    permission: "orders:read",
  },
  {
    label: "Склад",
    Icon: InventoryIcon,
    path: "/stock",
    permission: "stock:read",
  },
  {
    label: "Боты",
    Icon: SmartToyIcon,
    path: "/bots",
    permission: "referrals:read",
  },
  {
    label: "Роли",
    Icon: AdminPanelSettingsIcon,
    path: "/roles",
    permission: "rbac:manage",
  },
  {
    label: "Администраторы",
    Icon: GroupIcon,
    path: "/users",
    permission: "rbac:manage",
  },
  {
    label: "Изображения",
    Icon: PhotoLibraryIcon,
    path: "/images",
    permission: "images:upload", // Or a more general images permission
  },
  {
    label: "Журнал аудита",
    Icon: HistoryIcon,
    path: "/audit-log",
    permission: "audit_log.read",
  },
  {
    label: "Настройки",
    Icon: SettingsIcon,
    path: "/settings",
    permission: "settings:read",
  },
];
