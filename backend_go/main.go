package main

import (
	"log"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/routers"

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
	r.Run() // listen and serve on 0.0.0.0:8080
}
