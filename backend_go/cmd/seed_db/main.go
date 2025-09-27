package main

import (
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
	// Инициализируем генератор случайных чисел
	rand.New(rand.NewSource(time.Now().UnixNano()))

	// Загружаем конфигурацию
	appSettings, err := config.LoadConfig(".env.example")
	if err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	// Подключаемся к базе данных
	db, err := db.InitDB(appSettings)
	if err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	fmt.Println("Starting to seed the database...")
	seedData(db)
	fmt.Println("Database seeding completed successfully!")
}

func seedData(db *gorm.DB) {
	// ... (остальной код остается таким же)
	dropAllTables(db)
	autoMigrate(db)
	createAdmin(db)
	users := createBotUsers(db, 50)
	categories := createCategories(db)
	products := createProducts(db, categories, 100)
	createInitialStock(db, products)
	createOrdersAndTransactions(db, users, products, 200)
}

func dropAllTables(db *gorm.DB) {
	fmt.Println("Dropping all tables...")
	tables := []interface{}{&models.Transaction{}, &models.StockMovement{}, &models.Order{}, &models.Product{}, &models.Category{}, &models.BotUser{}, &models.User{}, &models.ReferralBot{}, &models.RefTransaction{}}
	if err := db.Migrator().DropTable(tables...); err != nil {
		log.Fatalf("Failed to drop tables: %v", err)
	}
}

func autoMigrate(db *gorm.DB) {
	fmt.Println("Auto-migrating tables...")
	if err := db.AutoMigrate(&models.User{}, &models.BotUser{}, &models.Category{}, &models.Product{}, &models.Order{}, &models.Transaction{}, &models.StockMovement{}, &models.ReferralBot{}, &models.RefTransaction{}); err != nil {
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
	var users []models.BotUser
	for i := 0; i < count; i++ {
		users = append(users, models.BotUser{
			TelegramID:       rand.Int63n(900000000) + 100000000, // Генерируем 9-значный ID
			HasPassedCaptcha: true,
		})
	}
	db.Create(&users)
	return users
}

func createCategories(db *gorm.DB) []models.Category {
	fmt.Println("Creating category tree...")

	// Уровень 1
	root1 := models.Category{Name: "Электроника"}
	root2 := models.Category{Name: "Книги"}
	root3 := models.Category{Name: "Одежда"}
	db.Create(&root1)
	db.Create(&root2)
	db.Create(&root3)

	// Уровень 2
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

	// Уровень 3
	child1_1_1 := models.Category{Name: "Apple", ParentID: &child1_1.ID}
	child1_1_2 := models.Category{Name: "Android", ParentID: &child1_1.ID}
	db.Create(&child1_1_1)
	db.Create(&child1_1_2)

	var allCategories []models.Category
	db.Find(&allCategories)
	return allCategories
}

func createProducts(db *gorm.DB, categories []models.Category, count int) []models.Product {
	fmt.Printf("Creating %d products in leaf categories...\n", count)

	// Находим ID всех категорий, которые являются родительскими
	parentIDs := make(map[uint]bool)
	for _, c := range categories {
		if c.ParentID != nil {
			parentIDs[*c.ParentID] = true
		}
	}

	// Собираем ID только тех категорий, которые не являются родительскими (конечные узлы)
	var leafCategoryIDs []uint
	for _, c := range categories {
		if !parentIDs[c.ID] {
			leafCategoryIDs = append(leafCategoryIDs, c.ID)
		}
	}

	fmt.Printf("Found %d leaf categories to assign products to.\n", len(leafCategoryIDs))

	var products []models.Product
	for i := 0; i < count; i++ {
		if len(leafCategoryIDs) == 0 {
			continue // На случай, если конечных категорий нет
		}
		products = append(products, models.Product{
			Name:       fmt.Sprintf("Product %d", i+1),
			Price:      float64(rand.Intn(50000-100) + 100),
			CategoryID: leafCategoryIDs[rand.Intn(len(leafCategoryIDs))],
			Details:    "{}",
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
			Quantity:    rand.Intn(91) + 10, // 10 to 100
			Description: "Initial stock",
		})
	}
	db.Create(&stockMovements)
}

func createOrdersAndTransactions(db *gorm.DB, users []models.BotUser, products []models.Product, count int) {
	fmt.Printf("Creating %d orders and transactions...\n", count)

	productMap := make(map[uint]models.Product)
	for _, p := range products {
		productMap[p.ID] = p
	}

	for i := 0; i < count; i++ {
		user := users[rand.Intn(len(users))]

		// Генерируем случайное время в пределах последнего месяца
		startTime := time.Now().AddDate(0, -1, 0).Unix()
		endTime := time.Now().Unix()
		randomTimestamp := rand.Int63n(endTime-startTime) + startTime
		created_at := time.Unix(randomTimestamp, 0)

		// 70% шанс на пополнение баланса
		if rand.Float32() < 0.7 {
			db.Create(&models.Transaction{
				UserID:      user.ID,
				Type:        models.Deposit,
				Amount:      float64(rand.Intn(10000-500) + 500),
				Description: "Test deposit",
				CreatedAt:   created_at,
			})
		} else {
			// 30% шанс на покупку
			product := products[rand.Intn(len(products))]
			quantity := rand.Intn(3) + 1
			amount := product.Price * float64(quantity)

			order := models.Order{
				UserID:    user.ID,
				ProductID: product.ID,
				Quantity:  quantity,
				Amount:    amount,
				Status:    "success",
				CreatedAt: created_at,
			}
			db.Create(&order)

			// Создаем транзакцию для заказа
			db.Create(&models.Transaction{
				UserID:      user.ID,
				OrderID:     &order.ID,
				Type:        models.Purchase,
				Amount:      -amount,
				Description: fmt.Sprintf("Purchase for order %d", order.ID),
				CreatedAt:   created_at,
			})

			// Создаем движение на складе для заказа
			db.Create(&models.StockMovement{
				OrderID:     &order.ID,
				ProductID:   product.ID,
				Type:        models.Sale,
				Quantity:    -quantity,
				Description: fmt.Sprintf("Sale for order %d", order.ID),
				CreatedAt:   created_at,
			})
		}
	}
}
