package routers

import (
	"net/http"
	"time"

	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
)

func (r *Router) AuthRouter(router *gin.Engine) {
	router.POST("/api/auth/login", r.loginHandler)
}

func (r *Router) loginHandler(c *gin.Context) {
	var form struct {
		Username string `form:"username"`
		Password string `form:"password"`
	}

	if err := c.ShouldBind(&form); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var user models.User
	r.db.Where("email = ?", form.Username).First(&user)

	if user.ID == 0 {
		errorResponse(c, http.StatusUnauthorized, "Incorrect username or password")
		return
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.HashedPassword), []byte(form.Password)); err != nil {
		errorResponse(c, http.StatusUnauthorized, "Incorrect username or password")
		return
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"sub":  user.Email,
		"role": user.Role,
		"exp":  time.Now().Add(time.Minute * time.Duration(r.appSettings.AccessTokenExpireMinutes)).Unix(),
	})

	tokenString, err := token.SignedString([]byte(r.appSettings.SecretKey))
	if err != nil {
		errorResponse(c, http.StatusInternalServerError, "Could not generate token")
		return
	}

	successResponse(c, http.StatusOK, gin.H{"access_token": tokenString, "token_type": "bearer"})
}
