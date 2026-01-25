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

	fmt.Println("Starting to seed the database with test data...")
	seedData(db, twoFAService)
	fmt.Println("Database seeding completed successfully!")
}

func seedData(db *gorm.DB, twoFAService services.TwoFAService) {
	permissions := getPermissions(db)
	createTestRoles(db)
	createTestUsers(db, permissions, twoFAService)
	users := createBotUsers(db, 50)
	categories := createCategories(db)
	products := createProducts(db, categories, 100)
	createInitialStock(db, products)
	createOrdersAndTransactions(db, users, products, 500)
}

func getPermissions(db *gorm.DB) map[string]models.Permission {
	var permissionsList []models.Permission
	db.Find(&permissionsList)
	permissionsMap := make(map[string]models.Permission)
	for _, p := range permissionsList {
		permissionsMap[p.Name] = p
	}
	return permissionsMap
}

func createTestRoles(db *gorm.DB) {
	fmt.Println("Creating test roles...")
	db.FirstOrCreate(&models.Role{Name: "бухгалтер"}, models.Role{Name: "бухгалтер"})
	db.FirstOrCreate(&models.Role{Name: "менеджер"}, models.Role{Name: "менеджер"})
	db.FirstOrCreate(&models.Role{Name: "godlike"}, models.Role{Name: "godlike"})
}

func assignUserRole(db *gorm.DB, user models.User, roleName string) {
	var role models.Role
	db.First(&role, "name = ?", roleName)
	if role.ID != 0 {
		userRole := models.UserRole{UserID: user.ID, RoleID: role.ID}
		db.FirstOrCreate(&userRole, userRole)
	}
}

func createTestUsers(db *gorm.DB, permissions map[string]models.Permission, twoFAService services.TwoFAService) {
	fmt.Println("Creating test users...")
	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte("password"), bcrypt.DefaultCost)

	// admin_dev user with admin role (static credentials for development)
	secretAdminDev := "QO4C6IF3RRNNUXLKAIVLOQPVYM5W3XEV" // Static 2FA secret for dev
	encryptedSecretAdminDev, _ := twoFAService.EncryptSecret(secretAdminDev)
	adminDevUser := models.User{Login: "admin_dev", HashedPassword: string(hashedPassword), IsActive: true, TwoFASecret: &encryptedSecretAdminDev}
	db.FirstOrCreate(&adminDevUser, models.User{Login: adminDevUser.Login})
	assignUserRole(db, adminDevUser, "admin")
	fmt.Printf("User created: admin_dev, 2FA Secret: %s\n", secretAdminDev)

	// admin_test user with admin role
	secretAdminTest, _ := twoFAService.GenerateSecret("admin_test")
	encryptedSecretAdminTest, _ := twoFAService.EncryptSecret(secretAdminTest)
	adminTestUser := models.User{Login: "admin_test", HashedPassword: string(hashedPassword), IsActive: true, TwoFASecret: &encryptedSecretAdminTest}
	db.FirstOrCreate(&adminTestUser, models.User{Login: adminTestUser.Login})
	assignUserRole(db, adminTestUser, "admin")
	fmt.Printf("User created: admin_test, 2FA Secret: %s\n", secretAdminTest)

	// Accountant
	secretAccountant, _ := twoFAService.GenerateSecret("accountant_test")
	encryptedSecretAccountant, _ := twoFAService.EncryptSecret(secretAccountant)
	accountant := models.User{Login: "accountant_test", HashedPassword: string(hashedPassword), IsActive: true, TwoFASecret: &encryptedSecretAccountant}
	db.FirstOrCreate(&accountant, models.User{Login: accountant.Login})
	var accountantRole models.Role
	db.First(&accountantRole, "name = ?", "бухгалтер")
	if accountantRole.ID != 0 {
		db.Create(&models.UserRole{UserID: accountant.ID, RoleID: accountantRole.ID})
		db.Create(&models.RolePermission{RoleID: accountantRole.ID, PermissionID: permissions["transactions:read"].ID})
		db.Create(&models.RolePermission{RoleID: accountantRole.ID, PermissionID: permissions["store_balance:read"].ID})
	}
	fmt.Printf("User created: accountant_test, 2FA Secret: %s\n", secretAccountant)

	// Manager
	secretManager, _ := twoFAService.GenerateSecret("manager_test")
	encryptedSecretManager, _ := twoFAService.EncryptSecret(secretManager)
	manager := models.User{Login: "manager_test", HashedPassword: string(hashedPassword), IsActive: true, TwoFASecret: &encryptedSecretManager}
	db.FirstOrCreate(&manager, models.User{Login: manager.Login})
	var managerRole models.Role
	db.First(&managerRole, "name = ?", "менеджер")
	if managerRole.ID != 0 {
		db.Create(&models.UserRole{UserID: manager.ID, RoleID: managerRole.ID})
		managerPermissions := []string{
			"products:read", "products:create", "products:update", "products:delete",
			"categories:read", "categories:create", "categories:update", "categories:delete",
			"orders:read", "stock:read", "stock:update",
		}
		for _, pName := range managerPermissions {
			db.Create(&models.RolePermission{RoleID: managerRole.ID, PermissionID: permissions[pName].ID})
		}
	}
	fmt.Printf("User created: manager_test, 2FA Secret: %s\n", secretManager)

	// Godlike user
	secretGod, _ := twoFAService.GenerateSecret("god_test")
	encryptedSecretGod, _ := twoFAService.EncryptSecret(secretGod)
	godlikeUser := models.User{Login: "god_test", HashedPassword: string(hashedPassword), IsActive: true, TwoFASecret: &encryptedSecretGod}
	db.FirstOrCreate(&godlikeUser, models.User{Login: godlikeUser.Login})
	var godlikeRole models.Role
	db.First(&godlikeRole, "name = ?", "godlike")
	if godlikeRole.ID != 0 {
		db.Create(&models.UserRole{UserID: godlikeUser.ID, RoleID: godlikeRole.ID})
		for _, p := range permissions {
			db.Create(&models.RolePermission{RoleID: godlikeRole.ID, PermissionID: p.ID})
		}
	}
	fmt.Printf("User created: god_test, 2FA Secret: %s\n", secretGod)
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
	totalStoreBalanceDelta := 0.0

	gateways := []struct {
		Name          string
		CommissionBPS int
	}{
		{"mock_provider", 500}, // 5%
		{"platform_card", 2000}, // 20%
		{"platform_sbp", 2000},  // 20%
	}
	platformCommissionBPS := 100 // 1%

	fmt.Println("Phase 1: Creating initial deposits for all users.")
	for _, user := range users {
		numDeposits := rand.Intn(4) + 2
		totalDeposit := 0.0
		for i := 0; i < numDeposits; i++ {
			depositAmount := float64(rand.Intn(15000-500) + 500)
			totalDeposit += depositAmount
			createdAt := time.Now().AddDate(-1, 0, 0).Add(time.Hour * time.Duration(rand.Intn(365*24)))

			gateway := gateways[rand.Intn(len(gateways))]
			gatewayCommission := depositAmount * float64(gateway.CommissionBPS) / 10000.0
			platformCommission := depositAmount * float64(platformCommissionBPS) / 10000.0
			storeBalanceDelta := depositAmount - gatewayCommission - platformCommission
			totalStoreBalanceDelta += storeBalanceDelta

			allTransactions = append(allTransactions, models.Transaction{
				UserID:             user.ID,
				Type:               models.Deposit,
				Amount:             depositAmount,
				Description:        "Test deposit",
				CreatedAt:          createdAt,
				PaymentGateway:     gateway.Name,
				GatewayCommission:  gatewayCommission,
				PlatformCommission: platformCommission,
				StoreBalanceDelta:  storeBalanceDelta,
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
				UserID:            user.ID,
				Type:              models.Purchase,
				Amount:            -product.Price,
				Description:       fmt.Sprintf("Purchase of %s", product.Name),
				CreatedAt:         createdAt,
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

	fmt.Println("Phase 4: Calculating and setting final store balance.")
	var finalStoreBalance float64
	db.Model(&models.Transaction{}).Select("sum(store_balance_delta)").Row().Scan(&finalStoreBalance)

	balance := models.StoreBalance{}
	db.First(&balance, 1)
	balance.CurrentBalance += finalStoreBalance
	if err := db.Save(&balance).Error; err != nil {
		log.Fatalf("Failed to create and set store balance: %v", err)
	}
	fmt.Printf("Final store balance set to: %.2f\n", finalStoreBalance)
}