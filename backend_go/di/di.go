package di

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/external_providers/contms"
	"frbktg/backend_go/gateways"
	"frbktg/backend_go/gateways/mock"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/repositories"
	"frbktg/backend_go/services"
	"frbktg/backend_go/workers"
	"log/slog"

	"gorm.io/gorm"
)

// Container holds all the dependencies of the application.
type Container struct {
	DB                     *gorm.DB
	AppSettings            config.Settings
	Logger                 *slog.Logger
	ProviderRegistry       *external_providers.ProviderRegistry
	PaymentGatewayRegistry *gateways.ProviderRegistry
	TokenService           services.TokenService
	AuthService            services.AuthService
	UserService            services.UserService
	ProductService         services.ProductService
	CategoryService        services.CategoryService
	ReferralService        services.ReferralService
	OrderService           services.OrderService
	TransactionService     services.TransactionService
	DashboardService       services.DashboardService
	BalanceService         services.BalanceService
	StockService           services.StockService
	AdminService           services.AdminService
	PaymentService         services.PaymentService
	WebhookService         services.WebhookService
	ImageService           services.ImageService
	SettingService         services.SettingService
	RoleService            services.RoleService
	UserRepo               repositories.UserRepository
	AuthHandler            *handlers.AuthHandler
	UserHandler            *handlers.UserHandler
	ProductHandler         *handlers.ProductHandler
	CategoryHandler        *handlers.CategoryHandler
	OrderHandler           *handlers.OrderHandler
	TransactionHandler     *handlers.TransactionHandler
	ReferralHandler        *handlers.ReferralHandler
	DashboardHandler       *handlers.DashboardHandler
	BalanceHandler         *handlers.BalanceHandler
	StockHandler           *handlers.StockHandler
	AdminHandler           *handlers.AdminHandler
	PaymentHandler         *handlers.PaymentHandler
	ImageHandler           *handlers.ImageHandler
	SettingHandler         *handlers.SettingHandler
	RoleHandler            *handlers.RoleHandler
	SubscriptionWorker     *workers.SubscriptionWorker
	PaymentWorker          *workers.PaymentWorker
}

// NewContainer creates a new dependency container.
func NewContainer(appSettings config.Settings) (*Container, error) {
	db, err := db.InitDB(appSettings)
	if err != nil {
		return nil, err
	}

	logger := slog.Default()

	// Init provider registry
	providerRegistry := external_providers.NewProviderRegistry()
	contmsAdapter := contms.NewContMSProxyAdapter("http://contms.ru:2525/api")
	providerRegistry.RegisterProvider(contmsAdapter)

	// Init payment gateway registry
	paymentGatewayRegistry := gateways.NewProviderRegistry()
	mockGatewayAdapter := mock.NewMockGatewayAdapter(appSettings.MockGatewayURL)
	paymentGatewayRegistry.RegisterProvider(mockGatewayAdapter)

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
	userSubscriptionRepo := repositories.NewUserSubscriptionRepository(db)
	paymentInvoiceRepo := repositories.NewPaymentInvoiceRepository(db)
	imageRepo := repositories.NewImageRepository(db)
	settingRepo := repositories.NewSettingRepository(db)
	roleRepo := repositories.NewRoleRepository(db)

	// Init services
	tokenService := services.NewTokenService(appSettings.SecretKey, appSettings.AccessTokenExpireMinutes)
	settingService := services.NewSettingService(settingRepo, userRepo)
	authService := services.NewAuthService(userRepo, tokenService, appSettings)
	userService := services.NewUserService(userRepo, botUserRepo, userSubscriptionRepo, orderRepo)
	productService := services.NewProductService(productRepo, categoryRepo, providerRegistry)
	categoryService := services.NewCategoryService(categoryRepo, productService)
	referralService := services.NewReferralService(userRepo, botUserRepo, referralRepo, transactionRepo, *settingService)
	orderService := services.NewOrderService(db, orderRepo, productRepo, botUserRepo, transactionRepo, userSubscriptionRepo, categoryRepo, referralService, providerRegistry)
	transactionService := services.NewTransactionService(transactionRepo)
	dashboardService := services.NewDashboardService(dashboardRepo)
	balanceService := services.NewBalanceService(balanceRepo, botUserRepo)
	stockService := services.NewStockService(stockRepo)
	adminService := services.NewAdminService(adminRepo, botUserRepo)
	webhookService := services.NewWebhookService(appSettings)
	paymentService := services.NewPaymentService(db, paymentGatewayRegistry, paymentInvoiceRepo, transactionRepo, botUserRepo, webhookService, appSettings)
	imageService := services.NewImageService(db, imageRepo, appSettings)
	roleService := services.NewRoleService(roleRepo)

	// Init workers
	subscriptionWorker := workers.NewSubscriptionWorker(orderService, userSubscriptionRepo, logger)
	paymentWorker := workers.NewPaymentWorker(paymentService, logger)

	// Init handlers
	authHandler := handlers.NewAuthHandler(authService)
	userHandler := handlers.NewUserHandler(userService, roleService)
	productHandler := handlers.NewProductHandler(productService)
	categoryHandler := handlers.NewCategoryHandler(categoryService)
	orderHandler := handlers.NewOrderHandler(orderService)
	transactionHandler := handlers.NewTransactionHandler(transactionService)
	referralHandler := handlers.NewReferralHandler(referralService)
	dashboardHandler := handlers.NewDashboardHandler(dashboardService)
	balanceHandler := handlers.NewBalanceHandler(balanceService)
	stockHandler := handlers.NewStockHandler(stockService)
	adminHandler := handlers.NewAdminHandler(adminService, userService)
	paymentHandler := handlers.NewPaymentHandler(paymentService)
	imageHandler := handlers.NewImageHandler(imageService)
	settingHandler := handlers.NewSettingHandler(settingService)
	roleHandler := handlers.NewRoleHandler(roleService)

	return &Container{
		DB:                     db,
		AppSettings:            appSettings,
		Logger:                 logger,
		ProviderRegistry:       providerRegistry,
		PaymentGatewayRegistry: paymentGatewayRegistry,
		TokenService:           tokenService,
		AuthService:            authService,
		UserService:            userService,
		ProductService:         productService,
		CategoryService:        categoryService,
		ReferralService:        referralService,
		OrderService:           orderService,
		TransactionService:     transactionService,
		DashboardService:       dashboardService,
		BalanceService:         balanceService,
		StockService:           stockService,
		AdminService:           adminService,
		PaymentService:         paymentService,
		WebhookService:         webhookService,
		ImageService:           imageService,
		RoleService:            roleService,
		UserRepo:               userRepo,
		AuthHandler:            authHandler,
		UserHandler:            userHandler,
		ProductHandler:         productHandler,
		CategoryHandler:        categoryHandler,
		OrderHandler:           orderHandler,
		TransactionHandler:     transactionHandler,
		ReferralHandler:        referralHandler,
		DashboardHandler:       dashboardHandler,
		BalanceHandler:         balanceHandler,
		StockHandler:           stockHandler,
		AdminHandler:           adminHandler,
		PaymentHandler:         paymentHandler,
		ImageHandler:           imageHandler,
		SettingHandler:         settingHandler,
		RoleHandler:            roleHandler,
		SubscriptionWorker:     subscriptionWorker,
		PaymentWorker:          paymentWorker,
	}, nil
}

// StartWorkers starts all the background workers.
func (c *Container) StartWorkers() {
	c.SubscriptionWorker.Start()
	c.PaymentWorker.Start()
}
