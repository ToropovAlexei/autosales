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
import AccountBalanceWalletIcon from "@mui/icons-material/AccountBalanceWallet";
import PriceCheckIcon from "@mui/icons-material/PriceCheck";
import CampaignIcon from "@mui/icons-material/Campaign";
import { AppRoute } from "@/types";

export const MENU_ITEMS = [
  { label: "Дашборд", Icon: HomeRoundedIcon, route: AppRoute.Dashboard },
  { label: "Категории", Icon: CategoryIcon, route: AppRoute.Categories },
  { label: "Ценообразование", Icon: PriceCheckIcon, route: AppRoute.Pricing },
  { label: "Товары", Icon: ShoppingCartIcon, route: AppRoute.Products },
  { label: "Покупатели", Icon: GroupIcon, route: AppRoute.BotUsers },
  {
    label: "Управление рекламой",
    Icon: CampaignIcon,
    route: AppRoute.Broadcasts,
  },
  { label: "Транзакции", Icon: ReceiptIcon, route: AppRoute.Transactions },
  { label: "Покупки", Icon: LocalMallIcon, route: AppRoute.Orders },
  { label: "Склад", Icon: InventoryIcon, route: AppRoute.Stock },
  { label: "Боты", Icon: SmartToyIcon, route: AppRoute.Bots },
  { label: "Роли", Icon: AdminPanelSettingsIcon, route: AppRoute.Roles },
  { label: "Администраторы", Icon: GroupIcon, route: AppRoute.Users },
  { label: "Изображения", Icon: PhotoLibraryIcon, route: AppRoute.Images },
  { label: "Журнал аудита", Icon: HistoryIcon, route: AppRoute.AuditLog },
  {
    label: "Приветственные сообщения",
    Icon: SettingsIcon,
    route: AppRoute.WelcomeMessages,
  },
  {
    label: "Управление рефералами",
    Icon: SettingsIcon,
    route: AppRoute.ReferralManagement,
  },
  {
    label: "Управление Балансом",
    Icon: AccountBalanceWalletIcon,
    route: AppRoute.Balance,
  },
];
