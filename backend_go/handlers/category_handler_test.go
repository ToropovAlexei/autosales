package handlers

import (
	"bytes"
	"encoding/json"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// MockCategoryService is a mock of CategoryService interface
type MockCategoryService struct {
	mock.Mock
}

func (m *MockCategoryService) GetAll() ([]models.CategoryResponse, error) {
	args := m.Called()
	return args.Get(0).([]models.CategoryResponse), args.Error(1)
}

func (m *MockCategoryService) GetByID(id uint) (*models.CategoryResponse, error) {
	args := m.Called(id)
	return args.Get(0).(*models.CategoryResponse), args.Error(1)
}

func (m *MockCategoryService) Create(name string, parentID *uint) (*models.CategoryResponse, error) {
	args := m.Called(name, parentID)
	return args.Get(0).(*models.CategoryResponse), args.Error(1)
}

func (m *MockCategoryService) Update(id uint, name string, parentID *uint) (*models.CategoryResponse, error) {
	args := m.Called(id, name, parentID)
	return args.Get(0).(*models.CategoryResponse), args.Error(1)
}

func (m *MockCategoryService) Delete(id uint) error {
	args := m.Called(id)
	return args.Error(0)
}

func TestCategoryHandler_GetCategoriesHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockService := new(MockCategoryService)
		mockCategories := []models.CategoryResponse{{ID: 1, Name: "Test Category"}}
		mockService.On("GetAll").Return(mockCategories, nil)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)

		h := NewCategoryHandler(mockService)
		router.GET("/categories", h.GetCategoriesHandler)

		req, _ := http.NewRequest(http.MethodGet, "/categories", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockService.AssertExpectations(t)
	})

	t.Run("Error", func(t *testing.T) {
		mockService := new(MockCategoryService)
		mockService.On("GetAll").Return([]models.CategoryResponse{}, apperrors.New(500, "DB error", nil))

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(500, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.GET("/categories", h.GetCategoriesHandler)

		req, _ := http.NewRequest(http.MethodGet, "/categories", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusInternalServerError, rr.Code)
		mockService.AssertExpectations(t)
	})
}

func TestCategoryHandler_CreateCategoryHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockService := new(MockCategoryService)
		categoryName := "New Category"
		mockCategory := &models.CategoryResponse{ID: 1, Name: categoryName}
		mockService.On("Create", categoryName, (*uint)(nil)).Return(mockCategory, nil)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)

		h := NewCategoryHandler(mockService)
		router.POST("/categories", h.CreateCategoryHandler)

		payload := map[string]string{"name": categoryName}
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPost, "/categories", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusCreated, rr.Code)
		mockService.AssertExpectations(t)
	})

	t.Run("Invalid Payload", func(t *testing.T) {
		mockService := new(MockCategoryService)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(400, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.POST("/categories", h.CreateCategoryHandler)

		payload := map[string]string{"name": ""} // Invalid payload
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPost, "/categories", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusBadRequest, rr.Code)
	})
}

func TestCategoryHandler_GetCategoryHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockService := new(MockCategoryService)
		mockCategory := &models.CategoryResponse{ID: 1, Name: "Test Category"}
		mockService.On("GetByID", uint(1)).Return(mockCategory, nil)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)

		h := NewCategoryHandler(mockService)
		router.GET("/categories/:id", h.GetCategoryHandler)

		req, _ := http.NewRequest(http.MethodGet, "/categories/1", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockService.AssertExpectations(t)
	})

	t.Run("Not Found", func(t *testing.T) {
		mockService := new(MockCategoryService)
		mockService.On("GetByID", uint(1)).Return((*models.CategoryResponse)(nil), &apperrors.ErrNotFound{Resource: "Category", ID: 1})

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(404, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.GET("/categories/:id", h.GetCategoryHandler)

		req, _ := http.NewRequest(http.MethodGet, "/categories/1", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusNotFound, rr.Code)
		mockService.AssertExpectations(t)
	})

	t.Run("Invalid ID", func(t *testing.T) {
		mockService := new(MockCategoryService)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(400, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.GET("/categories/:id", h.GetCategoryHandler)

		req, _ := http.NewRequest(http.MethodGet, "/categories/abc", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusBadRequest, rr.Code)
	})
}

func TestCategoryHandler_UpdateCategoryHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockService := new(MockCategoryService)
		categoryName := "Updated Category"
		mockCategory := &models.CategoryResponse{ID: 1, Name: categoryName}
		mockService.On("Update", uint(1), categoryName, (*uint)(nil)).Return(mockCategory, nil)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)

		h := NewCategoryHandler(mockService)
		router.PUT("/categories/:id", h.UpdateCategoryHandler)

		payload := map[string]string{"name": categoryName}
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPut, "/categories/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockService.AssertExpectations(t)
	})

	t.Run("Not Found", func(t *testing.T) {
		mockService := new(MockCategoryService)
		categoryName := "Updated Category"
		mockService.On("Update", uint(1), categoryName, (*uint)(nil)).Return((*models.CategoryResponse)(nil), &apperrors.ErrNotFound{Resource: "Category", ID: 1})

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(404, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.PUT("/categories/:id", h.UpdateCategoryHandler)

		payload := map[string]string{"name": categoryName}
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPut, "/categories/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusNotFound, rr.Code)
		mockService.AssertExpectations(t)
	})

	t.Run("Invalid ID", func(t *testing.T) {
		mockService := new(MockCategoryService)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(400, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.PUT("/categories/:id", h.UpdateCategoryHandler)

		req, _ := http.NewRequest(http.MethodPut, "/categories/abc", nil)
		req.Header.Set("Content-Type", "application/json")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusBadRequest, rr.Code)
	})

	t.Run("Invalid Payload", func(t *testing.T) {
		mockService := new(MockCategoryService)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(400, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.PUT("/categories/:id", h.UpdateCategoryHandler)

		payload := map[string]string{"name": ""} // Invalid payload
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPut, "/categories/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusBadRequest, rr.Code)
	})
}

func TestCategoryHandler_DeleteCategoryHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockService := new(MockCategoryService)
		mockService.On("Delete", uint(1)).Return(nil)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)

		h := NewCategoryHandler(mockService)
		router.DELETE("/categories/:id", h.DeleteCategoryHandler)

		req, _ := http.NewRequest(http.MethodDelete, "/categories/1", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusNoContent, rr.Code)
		mockService.AssertExpectations(t)
	})

	t.Run("Not Found", func(t *testing.T) {
		mockService := new(MockCategoryService)
		mockService.On("Delete", uint(1)).Return(&apperrors.ErrNotFound{Resource: "Category", ID: 1})

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(404, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.DELETE("/categories/:id", h.DeleteCategoryHandler)

		req, _ := http.NewRequest(http.MethodDelete, "/categories/1", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusNotFound, rr.Code)
		mockService.AssertExpectations(t)
	})

	t.Run("Invalid ID", func(t *testing.T) {
		mockService := new(MockCategoryService)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(400, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewCategoryHandler(mockService)
		router.DELETE("/categories/:id", h.DeleteCategoryHandler)

		req, _ := http.NewRequest(http.MethodDelete, "/categories/abc", nil)
		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusBadRequest, rr.Code)
	})
}