package handlers

import (
	"bytes"
	"encoding/json"
	"frbktg/backend_go/models"
	"frbktg/backend_go/services/mocks"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)



func TestProductHandler_GetProductsHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockProductService := new(mocks.MockProductService)
		products := []models.ProductResponse{{ID: 1, Name: "Test Product"}}

		mockProductService.On("GetProducts", []string(nil)).Return(products, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)

		h := NewProductHandler(mockProductService)
		h.GetProductsHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockProductService.AssertExpectations(t)
	})
}

func TestProductHandler_CreateProductHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockProductService := new(mocks.MockProductService)
		product := &models.ProductResponse{ID: 1, Name: "Test Product"}

		mockProductService.On("CreateProduct", "Test Product", uint(1), 10.0, 100).Return(product, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)

		payload := map[string]interface{}{"name": "Test Product", "category_id": 1, "price": 10.0, "initial_stock": 100}
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPost, "/products", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		c.Request = req

		h := NewProductHandler(mockProductService)
		h.CreateProductHandler(c)

		assert.Equal(t, http.StatusCreated, rr.Code)
		mockProductService.AssertExpectations(t)
	})
}

func TestProductHandler_GetProductHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockProductService := new(mocks.MockProductService)
		product := &models.ProductResponse{ID: 1, Name: "Test Product"}

		mockProductService.On("GetProduct", uint(1)).Return(product, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)
		c.Params = []gin.Param{{Key: "id", Value: "1"}}

		h := NewProductHandler(mockProductService)
		h.GetProductHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockProductService.AssertExpectations(t)
	})
}

func TestProductHandler_UpdateProductHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockProductService := new(mocks.MockProductService)
		product := &models.ProductResponse{ID: 1, Name: "Updated Product"}
		data := models.Product{Name: "Updated Product", CategoryID: 1, Price: 10.0}

		mockProductService.On("UpdateProduct", uint(1), data).Return(product, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)
		c.Params = []gin.Param{{Key: "id", Value: "1"}}

		payload := map[string]interface{}{"name": "Updated Product", "category_id": 1, "price": 10.0}
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPut, "/products/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		c.Request = req

		h := NewProductHandler(mockProductService)
		h.UpdateProductHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockProductService.AssertExpectations(t)
	})
}

func TestProductHandler_DeleteProductHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockProductService := new(mocks.MockProductService)

		mockProductService.On("DeleteProduct", uint(1)).Return(nil)

		rr := httptest.NewRecorder()
		c, router := gin.CreateTestContext(rr)
		router.Use(ErrorHandler())
		c.Params = []gin.Param{{Key: "id", Value: "1"}}

		h := NewProductHandler(mockProductService)
		router.DELETE("/products/:id", h.DeleteProductHandler)

		req, _ := http.NewRequest(http.MethodDelete, "/products/1", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusNoContent, rr.Code)
		mockProductService.AssertExpectations(t)
	})
}
