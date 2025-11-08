package handlers

import (
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

type ImageHandler struct {
	imageService services.ImageService
}

func NewImageHandler(imageService services.ImageService) *ImageHandler {
	return &ImageHandler{imageService: imageService}
}

// UploadImageHandler godoc
// @Summary      Upload an image
// @Description  Uploads a new image, checks for duplicates via hash.
// @Tags         Admin
// @Accept       multipart/form-data
// @Produce      json
// @Param        image formData file true "Image file to upload"
// @Param        folder_id formData int false "Folder ID"
// @Success      200  {object}  models.Image
// @Failure      400  {object}  apperrors.ErrorResponse
// @Failure      500  {object}  apperrors.ErrorResponse
// @Router       /admin/images/upload [post]
// @Security     ApiKeyAuth
func (h *ImageHandler) UploadImageHandler(c *gin.Context) {
	file, err := c.FormFile("image")
	if err != nil {
		c.Error(err)
		return
	}

	folder := c.PostForm("folder")

	image, err := h.imageService.UploadImage(file, folder)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, image)
}

// ListImagesHandler godoc
// @Summary      List all images
// @Description  Retrieves a list of all uploaded images, optionally filtered by folder.
// @Tags         Admin
// @Produce      json
// @Param        folder query string false "Filter by folder name"
// @Success      200  {array}  models.Image
// @Failure      500  {object}  apperrors.ErrorResponse
// @Router       /admin/images [get]
// @Security     ApiKeyAuth
func (h *ImageHandler) ListImagesHandler(c *gin.Context) {
	folder := c.Query("folder")

	images, err := h.imageService.ListImages(folder)
	if err != nil {
		c.Error(err)
		return
	}

	response := gin.H{
		"data":  images,
		"total": len(images),
	}

	responses.SuccessResponse(c, http.StatusOK, response)
}

// ServeImageHandler godoc
// @Summary      Serve an image
// @Description  Serves an image file by its UUID.
// @Tags         Public
// @Produce      image/png, image/jpeg, image/gif
// @Param        id   path      string  true  "Image UUID"
// @Success      200  {file}  string "Image file"
// @Failure      404  {object}  apperrors.ErrorResponse
// @Router       /images/{id} [get]
func (h *ImageHandler) ServeImageHandler(c *gin.Context) {
	idStr := c.Param("id")
	imageID, err := uuid.Parse(idStr)
	if err != nil {
		c.Error(err)
		return
	}

	filePath, err := h.imageService.GetImageFilePath(imageID)
	if err != nil {
		c.Error(err)
		return
	}

	c.File(filePath)
}

// DeleteImageHandler godoc
// @Summary      Delete an image
// @Description  Deletes an image by its UUID.
// @Tags         Admin
// @Produce      json
// @Param        id   path      string  true  "Image UUID"
// @Success      204  {object}  nil
// @Failure      404  {object}  apperrors.ErrorResponse
// @Failure      500  {object}  apperrors.ErrorResponse
// @Router       /admin/images/{id} [delete]
// @Security     ApiKeyAuth
func (h *ImageHandler) DeleteImageHandler(c *gin.Context) {
	idStr := c.Param("id")
	imageID, err := uuid.Parse(idStr)
	if err != nil {
		c.Error(err)
		return
	}

	if err := h.imageService.DeleteImage(imageID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}