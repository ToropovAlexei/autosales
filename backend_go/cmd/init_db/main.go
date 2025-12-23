package main

import (
	"encoding/json"
	"fmt"
	"log"
	"log/slog"
	"math/rand"
	"net/http"
	"time"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/models"
	"frbktg/backend_go/services"

	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

func main() {
	rand.New(rand.NewSource(time.Now().UnixNano()))
	appSettings, err := config.LoadConfig()
	if err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	db, err := db.InitDB(appSettings)
	if err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	twoFAService, err := services.NewTwoFAService(appSettings.TFASecretKey)
	if err != nil {
		log.Fatalf("could not create 2FA service: %v", err)
	}

	logger := slog.Default()

	fmt.Println("Dropping all tables...")
	dropAllTables(db)

	fmt.Println("Auto-migrating tables...")
	autoMigrate(db)

	createMainBots(db, appSettings, logger)
	createRbacData(db)
	adminUser := createAdmin(db, twoFAService)
	assignAdminRole(db, adminUser)
	createDefaultSettings(db)

	fmt.Println("Database initialization completed successfully!")
}

func dropAllTables(db *gorm.DB) {
	tables := []interface{}{
		&models.UserSubscription{}, &models.RefTransaction{}, &models.Bot{},
		&models.StockMovement{}, &models.Order{}, &models.Transaction{},
		&models.Product{}, &models.Category{}, &models.BotUser{}, &models.User{},
		&models.PaymentInvoice{}, &models.Image{}, &models.UserPermission{},
		&models.UserRole{}, &models.RolePermission{}, &models.Permission{}, &models.Role{},
		&models.ActiveToken{},
		&models.TemporaryToken{},
		&models.StoreBalance{},
		&models.Setting{},
	}
	if err := db.Migrator().DropTable(tables...); err != nil {
		log.Fatalf("Failed to drop tables: %v", err)
	}
}

func autoMigrate(db *gorm.DB) {
	if err := db.AutoMigrate(
		&models.User{},
		&models.Category{},
		&models.Product{},
		&models.BotUser{},
		&models.Transaction{},
		&models.Order{},
		&models.StockMovement{},
		&models.Bot{},
		&models.RefTransaction{},
		&models.UserSubscription{},
		&models.PaymentInvoice{},
		&models.Image{},
		&models.Role{},
		&models.Permission{},
		&models.RolePermission{},
		&models.UserRole{},
		&models.UserPermission{},
		&models.ActiveToken{},
		&models.TemporaryToken{},
		&models.StoreBalance{},
		&models.Setting{},
	); err != nil {
		log.Fatalf("failed to migrate database: %v", err)
	}
}

func createDefaultSettings(db *gorm.DB) {
	fmt.Println("Creating default settings...")
	settings := []models.Setting{
		{Key: "support_message", Value: "Здравствуйте! Чем могу помочь? Наша служба поддержки работает с 9:00 до 18:00 по будням."}, 
		{Key: "new_user_welcome_message", Value: `Добро пожаловать, {username}!
					Я - ваш личный помощник для покупок. Здесь вы можете:
					- 🛍️ Смотреть каталог товаров
					- 💰 Пополнять баланс
					- 💳 Проверять свой счет
					Выберите действие в меню ниже:`},
		{Key: "returning_user_welcome_message", Value: "Добро пожаловать, {username}! Чем могу помочь?"},
		{Key: "GLOBAL_PRICE_MARKUP", Value: "10"},
		{Key: "GATEWAY_COMMISSION_mock_provider", Value: "5"},
		{Key: "GATEWAY_COMMISSION_platform_card", Value: "3"},
		{Key: "GATEWAY_COMMISSION_platform_sbp", Value: "2"},
		{Key: "PLATFORM_COMMISSION_PERCENTAGE", Value: "1.5"},
	}
	for _, setting := range settings {
		db.FirstOrCreate(&setting, models.Setting{Key: setting.Key})
	}
}

func getBotUsername(token string) (string, error) {
	if token == "" {
		return "", nil
	}
	resp, err := http.Get(fmt.Sprintf("https://api.telegram.org/bot%s/getMe", token))
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("telegram api returned non-200 status code: %d", resp.StatusCode)
	}

	var result struct {
		OK     bool `json:"ok"`
		Result struct {
			Username string `json:"username"`
		} `json:"result"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	if !result.OK {
		return "", fmt.Errorf("telegram api returned ok=false")
	}

	return result.Result.Username, nil
}

func createMainBots(db *gorm.DB, appSettings *config.Config, logger *slog.Logger) {
	logger.Info("Creating main and fallback bots...")

	if appSettings.MainBotToken != "" {
		username, err := getBotUsername(appSettings.MainBotToken)
		if err != nil {
			logger.Error("failed to get main bot username", "error", err)
			username = appSettings.MainBotName
		}

		bot := models.Bot{
			Token:     appSettings.MainBotToken,
			Username:  username,
			Type:      "main",
			IsPrimary: true,
		}
		db.FirstOrCreate(&bot, models.Bot{Token: bot.Token})
		logger.Info("Created main bot", "username", bot.Username)
	}

	if appSettings.FallbackBotToken != "" {
		username, err := getBotUsername(appSettings.FallbackBotToken)
		if err != nil {
			logger.Error("failed to get fallback bot username", "error", err)
			username = appSettings.FallbackBotName
		}
		bot := models.Bot{
			Token:     appSettings.FallbackBotToken,
			Username:  username,
			Type:      "main",
			IsPrimary: false,
		}
		db.FirstOrCreate(&bot, models.Bot{Token: bot.Token})
		logger.Info("Created fallback bot", "username", bot.Username)
	}
}

func createRbacData(db *gorm.DB) {
	fmt.Println("Creating permissions and admin role...")
	permissionsList := []models.Permission{
		{Name: "rbac:manage", Group: "RBAC"},
		{Name: "dashboard:read", Group: "Dashboard"},
		{Name: "pricing:read", Group: "Pricing"},
		{Name: "pricing:edit", Group: "Pricing"},
		{Name: "products:read", Group: "Products"}, {Name: "products:create", Group: "Products"}, {Name: "products:update", Group: "Products"}, {Name: "products:delete", Group: "Products"},
		{Name: "categories:read", Group: "Categories"}, {Name: "categories:create", Group: "Categories"}, {Name: "categories:update", Group: "Categories"}, {Name: "categories:delete", Group: "Categories"},
		{Name: "orders:read", Group: "Orders"},
		{Name: "users:read", Group: "Users"}, {Name: "users:create", Group: "Users"}, {Name: "users:update", Group: "Users"}, {Name: "users:delete", Group: "Users"},
		{Name: "settings:read", Group: "Settings"}, {Name: "settings:edit", Group: "Settings"},
		{Name: "images:read", Group: "Images"}, {Name: "images:upload", Group: "Images"}, {Name: "images:delete", Group: "Images"},
		{Name: "referrals:read", Group: "Referrals"}, {Name: "referrals:update", Group: "Referrals"},
		{Name: "transactions:read", Group: "Transactions"},
		{Name: "store_balance:read", Group: "Balance"},
		{Name: "stock:read", Group: "Stock"}, {Name: "stock:update", Group: "Stock"},
		{Name: "audit_log:read", Group: "AuditLog"},
		{Name: "store_balance:manage", Group: "Balance"},
		{Name: "broadcasts:manage", Group: "Broadcasts"},
	}
	for _, p := range permissionsList {
		db.FirstOrCreate(&p, models.Permission{Name: p.Name})
	}

	db.FirstOrCreate(&models.Role{Name: "admin", IsSuper: true}, models.Role{Name: "admin"})
}

func createAdmin(db *gorm.DB, twoFAService services.TwoFAService) models.User {
	fmt.Println("Creating admin user...")
	password := randomString(16)
	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)

	secret, _ := twoFAService.GenerateSecret("admin")
	encryptedSecret, _ := twoFAService.EncryptSecret(secret)

	admin := models.User{
		Login:          "admin",
		HashedPassword: string(hashedPassword),
		IsActive:       true,
		TwoFASecret:    &encryptedSecret,
	}
	db.FirstOrCreate(&admin, models.User{Login: admin.Login})

	fmt.Println("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!")
	fmt.Printf("Admin user created: admin, Password: %s\n, 2FA Secret: %s\n", password, secret)
	fmt.Println("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!")

	return admin
}

func randomString(length int) string {
	const charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
	b := make([]byte, length)
	for i := range b {
		b[i] = charset[rand.Intn(len(charset))]
	}
	return string(b)
}

func assignAdminRole(db *gorm.DB, user models.User) {
	fmt.Println("Assigning admin role...")
	var adminRole models.Role
	db.First(&adminRole, "name = ?", "admin")
	if adminRole.ID != 0 {
		userRole := models.UserRole{UserID: user.ID, RoleID: adminRole.ID}
		db.FirstOrCreate(&userRole, userRole)
	}
}