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
	referralRepo := repositories.NewReferralRepository(db)
	dashboardRepo := repositories.NewDashboardRepository(db)
	balanceRepo := repositories.NewBalanceRepository(db)
	stockRepo := repositories.NewStockRepository(db)
	adminRepo := repositories.NewAdminRepository(db)

	// Init services
	tokenService := services.NewTokenService()
	authService := services.NewAuthService(userRepo, tokenService, appSettings)
	userService := services.NewUserService(userRepo, botUserRepo)
	productService := services.NewProductService(productRepo)
	categoryService := services.NewCategoryService(categoryRepo)
	referralService := services.NewReferralService(userRepo, botUserRepo, referralRepo, transactionRepo)
	orderService := services.NewOrderService(db, orderRepo, productRepo, botUserRepo, transactionRepo, referralService)
	transactionService := services.NewTransactionService(transactionRepo)
	dashboardService := services.NewDashboardService(dashboardRepo)
	balanceService := services.NewBalanceService(balanceRepo, botUserRepo)
	stockService := services.NewStockService(stockRepo)
	adminService := services.NewAdminService(adminRepo, botUserRepo)

	// Init handlers
	authHandler := handlers.NewAuthHandler(authService)
	userHandler := handlers.NewUserHandler(userService)
	productHandler := handlers.NewProductHandler(productService)
	categoryHandler := handlers.NewCategoryHandler(categoryService)
	orderHandler := handlers.NewOrderHandler(orderService)
	transactionHandler := handlers.NewTransactionHandler(transactionService)
	referralHandler := handlers.NewReferralHandler(referralService)
	dashboardHandler := handlers.NewDashboardHandler(dashboardService)
	balanceHandler := handlers.NewBalanceHandler(balanceService)
	stockHandler := handlers.NewStockHandler(stockService)
	adminHandler := handlers.NewAdminHandler(adminService)

	r := gin.Default()

	corsConfig := cors.DefaultConfig()
	corsConfig.AllowOrigins = appSettings.CorsOrigins
	corsConfig.AllowCredentials = true
	corsConfig.AddAllowMethods("*")
	corsConfig.AddAllowHeaders("*")
	r.Use(cors.New(corsConfig))

	logger := slog.Default()
	rtr := routers.NewRouter(db, appSettings, logger, tokenService, userRepo)

	rtr.AuthRouter(r, authHandler)
	rtr.CategoriesRouter(r, categoryHandler)
	rtr.ProductsRouter(r, productHandler)
	rtr.UsersRouter(r, userHandler)
	rtr.BalanceRouter(r, balanceHandler)
	rtr.OrdersRouter(r, orderHandler)
	rtr.AdminRouter(r, adminHandler)
	rtr.TransactionsRouter(r, transactionHandler)
	rtr.StockRouter(r, stockHandler)
	rtr.DashboardRouter(r, dashboardHandler)
	rtr.ReferralsRouter(r, referralHandler)

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
