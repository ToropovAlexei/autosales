//go:generate /home/user/go/bin/swag init
package main

import (
	"flag"
	"net/http"
	"strings"

	"frbktg/backend_go/config"
	"frbktg/backend_go/di"
	_ "frbktg/backend_go/docs" // This is required for swag to find your docs
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"
	"frbktg/backend_go/routers"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
	"github.com/rs/zerolog/log"
)

// GinLogAdapter перенаправляет логи Gin в zerolog.
type GinLogAdapter struct{}

// Write анализирует сообщение от Gin и логирует его с подходящим уровнем.
func (gla GinLogAdapter) Write(p []byte) (n int, err error) {
	msg := strings.TrimSpace(string(p))
	if strings.HasPrefix(msg, "[WARNING]") {
		log.Warn().Msg(msg)
	} else {
		log.Debug().Msg(msg)
	}
	return len(p), nil
}

// @title           Your Project API
// @version         1.0
// @description     This is the API for your project.
// @termsOfService  http://swagger.io/terms/

// @contact.name   API Support
// @contact.url    http://www.swagger.io/support
// @contact.email  support@swagger.io

// @license.name  Apache 2.0
// @license.url   http://www.apache.org/licenses/LICENSE-2.0.html

// @host      localhost:8000
// @BasePath  /api

// @securityDefinitions.apiKey  ApiKeyAuth
// @in header
// @name Authorization

// @securityDefinitions.apiKey  ServiceApiKeyAuth
// @in header
// @name X-API-KEY
func main() {
	config.InitLogger()

	configPath := flag.String("config", ".env", "path to config file")
	flag.Parse()

	appSettings, err := config.LoadConfig(*configPath)
	if err != nil {
		log.Fatal().Err(err).Msg("could not load config")
	}

	container, err := di.NewContainer(appSettings)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to create container")
	}

	// Start workers
	container.StartWorkers()

	// Run migrations
	if err := container.DB.AutoMigrate(&models.Setting{}); err != nil {
		log.Fatal().Err(err).Msg("failed to migrate database")
	}

	// Перенаправляем стандартный логгер Gin в zerolog через наш адаптер
	gin.DefaultWriter = GinLogAdapter{}

	r := gin.New()
	r.RedirectTrailingSlash = false
	r.RedirectFixedPath = false

	// Используем стандартный Recovery middleware и наши кастомные
	r.Use(gin.Recovery())
	r.Use(middleware.LogContext())   // Добавляет контекст в логгер
	r.Use(middleware.ErrorHandler()) // Должен быть после LogContext

	corsConfig := cors.DefaultConfig()
	corsConfig.AllowOrigins = appSettings.CorsOrigins
	corsConfig.AllowCredentials = true
	corsConfig.AddAllowMethods("*")
	corsConfig.AddAllowHeaders("*")
	r.Use(cors.New(corsConfig))

	authMiddleware := middleware.NewAuthMiddleware(container.TokenService, container.UserService)

	// Register all routes
	routers.RegisterAuthRoutes(r, container.AuthHandler)
	routers.RegisterCategoryRoutes(r, container.CategoryHandler, authMiddleware)
	routers.RegisterProductRoutes(r, container.ProductHandler, authMiddleware)
	routers.RegisterUserRoutes(r, container.UserHandler, authMiddleware)
	routers.RegisterBalanceRoutes(r, container.BalanceHandler, authMiddleware)
	routers.RegisterOrderRoutes(r, container.OrderHandler, authMiddleware)
	routers.RegisterAdminRoutes(r, container.AdminHandler, authMiddleware)
	routers.RegisterTransactionRoutes(r, container.TransactionHandler, authMiddleware)
	routers.RegisterStockRoutes(r, container.StockHandler, authMiddleware)
	routers.RegisterDashboardRoutes(r, container.DashboardHandler, authMiddleware)
	routers.RegisterReferralRoutes(r, container.ReferralHandler, authMiddleware)
	routers.RegisterPaymentRoutes(r, container.PaymentHandler, authMiddleware)
	routers.RegisterImageRoutes(r, container.ImageHandler, authMiddleware)
	routers.RegisterSettingRoutes(r, container.SettingHandler, authMiddleware)

	// Swagger route
	// rtr.SwaggerRouter(r)

	for _, route := range r.Routes() {
		log.Info().Msgf("Registered route: %s %s", route.Method, route.Path)
	}

	r.GET("/api", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"message": "Welcome to the API",
		})
	})
	if runErr := r.Run(":" + appSettings.Port); runErr != nil {
		log.Fatal().Err(runErr).Msg("failed to run server")
	}
}
