package main

import (
	"database/sql"
	"fmt"
	"log"
	"math/rand"
	"time"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/models"

	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

func main() {
	rand.New(rand.NewSource(time.Now().UnixNano()))

	appSettings, err := config.LoadConfig(".env")
	if err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	db, err := db.InitDB(appSettings)
	if err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	fmt.Println("Starting to seed the database...")
	seedData(db)
	fmt.Println("Database seeding completed successfully!")
}

func seedData(db *gorm.DB) {
	dropAllTables(db)
	autoMigrate(db)
	permissions := createRbacData(db)
	adminUser := createAdmin(db)
	assignAdminRole(db, adminUser)
	createTestUsers(db, permissions)
	users := createBotUsers(db, 50)
	categories := createCategories(db)
	products := createProducts(db, categories, 100)
	createInitialStock(db, products)
	createOrdersAndTransactions(db, users, products, 500)
	createDefaultSettings(db)
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
		{Key: "returning_user_welcome_message", Value: `Добро пожаловать, {username}! Чем могу помочь?`},
	}
	for _, setting := range settings {
		db.FirstOrCreate(&setting, models.Setting{Key: setting.Key})
	}
}

func dropAllTables(db *gorm.DB) {
	fmt.Println("Dropping all tables...")
	tables := []interface{}{
		&models.UserSubscription{}, &models.RefTransaction{}, &models.ReferralBot{},
		&models.StockMovement{}, &models.Order{}, &models.Transaction{},
		&models.Product{}, &models.Category{}, &models.BotUser{}, &models.User{},
		&models.PaymentInvoice{}, &models.Image{}, &models.UserPermission{},
		&models.UserRole{}, &models.RolePermission{}, &models.Permission{}, &models.Role{},
		&models.ActiveToken{},
	}
	if err := db.Migrator().DropTable(tables...); err != nil {
		log.Fatalf("Failed to drop tables: %v", err)
	}
}

func autoMigrate(db *gorm.DB) {
	fmt.Println("Auto-migrating tables...")
	if err := db.AutoMigrate(
		&models.User{}, &models.BotUser{}, &models.Category{}, &models.Product{},
		&models.Order{}, &models.Transaction{}, &models.StockMovement{},
		&models.ReferralBot{}, &models.RefTransaction{}, &models.UserSubscription{},
		&models.PaymentInvoice{}, &models.Image{}, &models.Role{}, &models.Permission{},
		&models.RolePermission{}, &models.UserRole{}, &models.UserPermission{}, &models.Setting{},
		&models.ActiveToken{},
	); err != nil {
		log.Fatalf("Failed to auto-migrate: %v", err)
	}
}

func createRbacData(db *gorm.DB) map[string]models.Permission {
	fmt.Println("Creating permissions and roles...")
	permissionsList := []models.Permission{
		{Name: "rbac:manage", Group: "RBAC"},
		{Name: "products:read", Group: "Products"}, {Name: "products:create", Group: "Products"}, {Name: "products:update", Group: "Products"}, {Name: "products:delete", Group: "Products"},
		{Name: "categories:read", Group: "Categories"}, {Name: "categories:create", Group: "Categories"}, {Name: "categories:update", Group: "Categories"}, {Name: "categories:delete", Group: "Categories"},
		{Name: "orders:read", Group: "Orders"}, {Name: "orders:update", Group: "Orders"},
		{Name: "users:read", Group: "Users"}, {Name: "users:create", Group: "Users"}, {Name: "users:update", Group: "Users"}, {Name: "users:delete", Group: "Users"},
		{Name: "dashboard:read", Group: "Dashboard"},
		{Name: "settings:read", Group: "Settings"}, {Name: "settings:edit", Group: "Settings"},
		{Name: "images:read", Group: "Images"}, {Name: "images:upload", Group: "Images"}, {Name: "images:delete", Group: "Images"},
		{Name: "referrals:read", Group: "Referrals"}, {Name: "referrals:update", Group: "Referrals"},
		{Name: "transactions:read", Group: "Transactions"},
		{Name: "balance:read", Group: "Balance"},
		{Name: "stock:read", Group: "Stock"}, {Name: "stock:update", Group: "Stock"},
		{Name: "audit_log.read", Group: "AuditLog"},
	}
	permissionsMap := make(map[string]models.Permission)
	for _, p := range permissionsList {
		db.FirstOrCreate(&p, models.Permission{Name: p.Name})
		permissionsMap[p.Name] = p
	}

	db.FirstOrCreate(&models.Role{Name: "admin", IsSuper: true}, models.Role{Name: "admin"})
	db.FirstOrCreate(&models.Role{Name: "бухгалтер"}, models.Role{Name: "бухгалтер"})
	db.FirstOrCreate(&models.Role{Name: "менеджер"}, models.Role{Name: "менеджер"})

	return permissionsMap
}

func createAdmin(db *gorm.DB) models.User {
	fmt.Println("Creating admin user...")
	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte("password"), bcrypt.DefaultCost)
	admin := models.User{
		Email:          "test@example.com",
		HashedPassword: string(hashedPassword),
		IsActive:       true,
	}
	db.FirstOrCreate(&admin, models.User{Email: admin.Email})
	return admin
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

func createTestUsers(db *gorm.DB, permissions map[string]models.Permission) {
	fmt.Println("Creating test users...")
	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte("password"), bcrypt.DefaultCost)

	// admin@gmail.com user with admin role
	admin2 := models.User{Email: "admin@gmail.com", HashedPassword: string(hashedPassword), IsActive: true}
	db.FirstOrCreate(&admin2, models.User{Email: admin2.Email})
	assignAdminRole(db, admin2)

	// Accountant
	accountant := models.User{Email: "accountant@example.com", HashedPassword: string(hashedPassword), IsActive: true}
	db.FirstOrCreate(&accountant, models.User{Email: accountant.Email})
	var accountantRole models.Role
	db.First(&accountantRole, "name = ?", "бухгалтер")
	if accountantRole.ID != 0 {
		db.Create(&models.UserRole{UserID: accountant.ID, RoleID: accountantRole.ID})
		db.Create(&models.RolePermission{RoleID: accountantRole.ID, PermissionID: permissions["transactions:read"].ID})
		db.Create(&models.RolePermission{RoleID: accountantRole.ID, PermissionID: permissions["balance:read"].ID})
	}

	// Manager
	manager := models.User{Email: "manager@example.com", HashedPassword: string(hashedPassword), IsActive: true}
	db.FirstOrCreate(&manager, models.User{Email: manager.Email})
	var managerRole models.Role
	db.First(&managerRole, "name = ?", "менеджер")
	if managerRole.ID != 0 {
		db.Create(&models.UserRole{UserID: manager.ID, RoleID: managerRole.ID})
		managerPermissions := []string{
			"products:read", "products:create", "products:update", "products:delete",
			"categories:read", "categories:create", "categories:update", "categories:delete",
			"orders:read", "orders:update", "stock:read", "stock:update",
		}
		for _, pName := range managerPermissions {
			db.Create(&models.RolePermission{RoleID: managerRole.ID, PermissionID: permissions[pName].ID})
		}
	}

	// Godlike user
	godlikeUser := models.User{Email: "god@example.com", HashedPassword: string(hashedPassword), IsActive: true}
	db.FirstOrCreate(&godlikeUser, models.User{Email: godlikeUser.Email})
	godlikeRole := models.Role{Name: "godlike", IsSuper: false}
	db.FirstOrCreate(&godlikeRole, models.Role{Name: godlikeRole.Name})
	db.Create(&models.UserRole{UserID: godlikeUser.ID, RoleID: godlikeRole.ID})
	for _, p := range permissions {
		db.Create(&models.RolePermission{RoleID: godlikeRole.ID, PermissionID: p.ID})
	}
}

func createBotUsers(db *gorm.DB, count int) []models.BotUser {
	fmt.Printf("Creating %d bot users...\n", count)
	botNames := []string{"main_bot", "referral_bot_1", "promo_bot", "support_bot"}
	var users []models.BotUser
	for i := 0; i < count; i++ {
		botName := botNames[rand.Intn(len(botNames))]
		createdAt := time.Now().AddDate(0, -rand.Intn(12), -rand.Intn(28))
		lastSeen := createdAt.Add(time.Hour * time.Duration(rand.Intn(24*30)))
		users = append(users, models.BotUser{
			TelegramID:        rand.Int63n(900000000) + 100000000,
			HasPassedCaptcha:  true,
			RegisteredWithBot: botName,
			LastSeenWithBot:   botName,
			LastSeenAt:        lastSeen,
			CreatedAt:         createdAt,
		})
	}
	db.Create(&users)
	return users
}

func createCategories(db *gorm.DB) []models.Category {
	fmt.Println("Creating category tree...")
	root1 := models.Category{Name: "Электроника"}
	root2 := models.Category{Name: "Книги"}
	root3 := models.Category{Name: "Одежда"}
	db.Create(&root1)
	db.Create(&root2)
	db.Create(&root3)

	child1_1 := models.Category{Name: "Смартфоны", ParentID: &root1.ID}
	child1_2 := models.Category{Name: "Ноутбуки", ParentID: &root1.ID}
	db.Create(&child1_1)
	db.Create(&child1_2)

	child2_1 := models.Category{Name: "Художественная литература", ParentID: &root2.ID}
	db.Create(&child2_1)

	child3_1 := models.Category{Name: "Мужская", ParentID: &root3.ID}
	child3_2 := models.Category{Name: "Женская", ParentID: &root3.ID}
	db.Create(&child3_1)
	db.Create(&child3_2)

	child1_1_1 := models.Category{Name: "Apple", ParentID: &child1_1.ID}
	child1_1_2 := models.Category{Name: "Android", ParentID: &child1_1.ID}
	db.Create(&child1_1_1)
	db.Create(&child1_1_2)

	var allCategories []models.Category
	db.Find(&allCategories)
	return allCategories
}

func createProducts(db *gorm.DB, categories []models.Category, count int) []models.Product {
	fmt.Printf("Creating %d products...\n", count)
	parentIDs := make(map[uint]bool)
	for _, c := range categories {
		if c.ParentID != nil {
			parentIDs[*c.ParentID] = true
		}
	}

	var leafCategoryIDs []uint
	for _, c := range categories {
		if !parentIDs[c.ID] {
			leafCategoryIDs = append(leafCategoryIDs, c.ID)
		}
	}

	var products []models.Product
	for i := 0; i < count; i++ {
		if len(leafCategoryIDs) == 0 {
			continue
		}
		products = append(products, models.Product{
			Name:       fmt.Sprintf("Product %d", i+1),
			Price:      float64(rand.Intn(25000-100) + 100),
			CategoryID: leafCategoryIDs[rand.Intn(len(leafCategoryIDs))],
			Details:    sql.NullString{String: "{}", Valid: true},
		})
	}
	db.Create(&products)
	return products
}

func createInitialStock(db *gorm.DB, products []models.Product) {
	fmt.Println("Creating initial stock...")
	var stockMovements []models.StockMovement
	for _, p := range products {
		stockMovements = append(stockMovements, models.StockMovement{
			ProductID:   p.ID,
			Type:        models.Initial,
			Quantity:    rand.Intn(91) + 10,
			Description: "Initial stock",
		})
	}
	db.Create(&stockMovements)
}

func createOrdersAndTransactions(db *gorm.DB, users []models.BotUser, products []models.Product, purchaseCount int) {
	fmt.Println("Creating deposits and purchases...")
	userBalances := make(map[uint]float64)
	var allTransactions []models.Transaction

	fmt.Println("Phase 1: Creating initial deposits for all users.")
	for _, user := range users {
		numDeposits := rand.Intn(4) + 2
		totalDeposit := 0.0
		for i := 0; i < numDeposits; i++ {
			depositAmount := float64(rand.Intn(15000-500) + 500)
			totalDeposit += depositAmount
			createdAt := time.Now().AddDate(-1, 0, 0).Add(time.Hour * time.Duration(rand.Intn(365*24)))
			allTransactions = append(allTransactions, models.Transaction{
				UserID:      user.ID,
				Type:        models.Deposit,
				Amount:      depositAmount,
				Description: "Test deposit",
				CreatedAt:   createdAt,
			})
		}
		userBalances[user.ID] = totalDeposit
	}
	db.Create(&allTransactions)

	fmt.Printf("Phase 2: Creating %d valid purchases.\n", purchaseCount)
	var allPurchaseTransactions []models.Transaction
	var allStockMovements []models.StockMovement

	for i := 0; i < purchaseCount; i++ {
		user := users[rand.Intn(len(users))]
		product := products[rand.Intn(len(products))]
		quantity := 1
		amount := product.Price * float64(quantity)

		if userBalances[user.ID] >= amount {
			userBalances[user.ID] -= amount
			createdAt := time.Now().AddDate(-1, 0, 0).Add(time.Hour * time.Duration(rand.Intn(365*24)))
			order := models.Order{
				UserID:    user.ID,
				ProductID: product.ID,
				Quantity:  quantity,
				Amount:    amount,
				Status:    "success",
				CreatedAt: createdAt,
			}
			db.Create(&order)

			allPurchaseTransactions = append(allPurchaseTransactions, models.Transaction{
				UserID:      user.ID,
				OrderID:     &order.ID,
				Type:        models.Purchase,
				Amount:      -amount,
				Description: fmt.Sprintf("Purchase for order %d", order.ID),
				CreatedAt:   createdAt,
			})

			allStockMovements = append(allStockMovements, models.StockMovement{
				OrderID:     &order.ID,
				ProductID:   product.ID,
				Type:        models.Sale,
				Quantity:    -quantity,
				Description: fmt.Sprintf("Sale for order %d", order.ID),
				CreatedAt:   createdAt,
			})
		}
	}
	db.Create(&allPurchaseTransactions)
	db.Create(&allStockMovements)

	fmt.Println("Phase 3: Updating user balances.")
	for userID, balance := range userBalances {
		db.Model(&models.BotUser{}).Where("id = ?", userID).Update("balance", balance)
	}
}
