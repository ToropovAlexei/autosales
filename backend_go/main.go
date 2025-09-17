package main

import (
	"log"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/routers"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
)

func main() {
	if err := config.LoadConfig(".env.example"); err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	if err := db.InitDB(); err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	r := gin.Default()

	corsConfig := cors.DefaultConfig()
	corsConfig.AllowOrigins = config.AppSettings.CORS_ORIGINS
	corsConfig.AllowCredentials = true
	corsConfig.AddAllowMethods("*")
	corsConfig.AddAllowHeaders("*")
	r.Use(cors.New(corsConfig))

	routers.AuthRouter(r)
	routers.CategoriesRouter(r)
	routers.ProductsRouter(r)
	routers.UsersRouter(r)
	routers.BalanceRouter(r)
	routers.OrdersRouter(r)
	routers.AdminRouter(r)
	routers.TransactionsRouter(r)
	routers.StockRouter(r)
	routers.DashboardRouter(r)
	routers.ReferralsRouter(r)

	for _, route := range r.Routes() {
		log.Printf("Registered route: %s %s", route.Method, route.Path)
	}

	r.GET("/api", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"message": "Welcome to the API",
		})
	})
	if err := r.Run(":" + config.AppSettings.PORT); err != nil {
		log.Fatalf("failed to run server: %v", err)
	}
}
