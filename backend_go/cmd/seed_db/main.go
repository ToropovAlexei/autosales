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

	appSettings, err := config.LoadConfig(".env.example")
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
	createAdmin(db)
	users := createBotUsers(db, 50)
	categories := createCategories(db)
	products := createProducts(db, categories, 100)
	createInitialStock(db, products)
	createOrdersAndTransactions(db, users, products, 500) // Increased transaction count
}

func dropAllTables(db *gorm.DB) {
	fmt.Println("Dropping all tables...")
	tables := []interface{}{&models.UserSubscription{}, &models.RefTransaction{}, &models.ReferralBot{}, &models.StockMovement{}, &models.Order{}, &models.Transaction{}, &models.Product{}, &models.Category{}, &models.BotUser{}, &models.User{}, &models.PaymentInvoice{}}
	if err := db.Migrator().DropTable(tables...); err != nil {
		log.Fatalf("Failed to drop tables: %v", err)
	}
}

func autoMigrate(db *gorm.DB) {
	fmt.Println("Auto-migrating tables...")
	if err := db.AutoMigrate(&models.User{}, &models.BotUser{}, &models.Category{}, &models.Product{}, &models.Order{}, &models.Transaction{}, &models.StockMovement{}, &models.ReferralBot{}, &models.RefTransaction{}, &models.UserSubscription{}, &models.PaymentInvoice{}); err != nil {
		log.Fatalf("Failed to auto-migrate: %v", err)
	}
}

func createAdmin(db *gorm.DB) {
	fmt.Println("Creating admin user...")
	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte("password"), bcrypt.DefaultCost)
	admin := models.User{Email: "test@example.com", HashedPassword: string(hashedPassword), Role: models.Admin}
	db.Create(&admin)
}

func createBotUsers(db *gorm.DB, count int) []models.BotUser {
	fmt.Printf("Creating %d bot users...\n", count)
	botNames := []string{"main_bot", "referral_bot_1", "promo_bot", "support_bot"}
	var users []models.BotUser
	for i := 0; i < count; i++ {
		botName := botNames[rand.Intn(len(botNames))]
		// Random time in the last year
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

	// 1. Create initial deposits for every user
	fmt.Println("Phase 1: Creating initial deposits for all users.")
	for _, user := range users {
		numDeposits := rand.Intn(4) + 2 // 2 to 5 deposits per user
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

	// 2. Create purchases, ensuring user has enough balance
	fmt.Printf("Phase 2: Creating %d valid purchases.\n", purchaseCount)
	var allPurchaseTransactions []models.Transaction
	var allStockMovements []models.StockMovement

	for i := 0; i < purchaseCount; i++ {
		user := users[rand.Intn(len(users))]
		product := products[rand.Intn(len(products))]
		quantity := 1 // Keep quantity simple
		amount := product.Price * float64(quantity)

		if userBalances[user.ID] >= amount {
			// User can afford it, create the purchase
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
			// We need to create order one by one to get its ID for FKs
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
}