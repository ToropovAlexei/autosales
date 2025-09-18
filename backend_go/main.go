package main

import (
	"log"
	"log/slog"
	"net/http"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/repositories"
	"frbktg/backend_go/routers"
	"frbktg/backend_go/services"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
)

func main() {
	appSettings, err := config.LoadConfig(".env.example")
	if err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	db, err := db.InitDB(appSettings)
	if err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	// Init repositories
	userRepo := repositories.NewUserRepository(db)
	botUserRepo := repositories.NewBotUserRepository(db)
	productRepo := repositories.NewProductRepository(db)
	categoryRepo := repositories.NewCategoryRepository(db)
	orderRepo := repositories.NewOrderRepository(db)
	transactionRepo := repositories.NewTransactionRepository(db)

	// Init services
	userService := services.NewUserService(userRepo, botUserRepo)
	productService := services.NewProductService(productRepo)
	categoryService := services.NewCategoryService(categoryRepo)
	orderService := services.NewOrderService(db, orderRepo, productRepo, botUserRepo, transactionRepo)
	transactionService := services.NewTransactionService(transactionRepo)

	// Init handlers
	userHandler := handlers.NewUserHandler(userService)
	productHandler := handlers.NewProductHandler(productService)
	categoryHandler := handlers.NewCategoryHandler(categoryService)
	orderHandler := handlers.NewOrderHandler(orderService)
	transactionHandler := handlers.NewTransactionHandler(transactionService)

	r := gin.Default()

	corsConfig := cors.DefaultConfig()
	corsConfig.AllowOrigins = appSettings.CorsOrigins
	corsConfig.AllowCredentials = true
	corsConfig.AddAllowMethods("*")
	corsConfig.AddAllowHeaders("*")
	r.Use(cors.New(corsConfig))

	logger := slog.Default()
	rtr := routers.NewRouter(db, appSettings, logger)

	rtr.AuthRouter(r)
	rtr.CategoriesRouter(r, categoryHandler)
	rtr.ProductsRouter(r, productHandler)
	rtr.UsersRouter(r, userHandler)
	rtr.BalanceRouter(r)
	rtr.OrdersRouter(r, orderHandler)
	rtr.AdminRouter(r)
	rtr.TransactionsRouter(r, transactionHandler)
	rtr.StockRouter(r)
	rtr.DashboardRouter(r)
	rtr.ReferralsRouter(r)

	for _, route := range r.Routes() {
		log.Printf("Registered route: %s %s", route.Method, route.Path)
	}

	r.GET("/api", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"message": "Welcome to the API",
		})
	})
	if runErr := r.Run(":" + appSettings.Port); runErr != nil {
		log.Fatalf("failed to run server: %v", runErr)
	}
}
