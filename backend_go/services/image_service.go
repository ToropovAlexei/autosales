package services

import (
	"crypto/sha256"
	"errors"
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/config"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"io"
	"mime/multipart"
	"os"
	"path/filepath"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type ImageService interface {
	ListImages(folder string) ([]models.Image, error)
	UploadImage(fileHeader *multipart.FileHeader, folder string) (*models.Image, error)
	GetImageFilePath(imageID uuid.UUID) (string, error)
	DeleteImage(imageID uuid.UUID) error
}

type imageService struct {
	db        *gorm.DB
	imageRepo repositories.ImageRepository
	cfg       *config.Config
}

func NewImageService(db *gorm.DB, imageRepo repositories.ImageRepository, cfg *config.Config) ImageService {
	// Ensure the upload directory exists
	if err := os.MkdirAll(cfg.ImageUploadPath, os.ModePerm); err != nil {
		panic(fmt.Sprintf("failed to create image upload directory: %v", err))
	}
	return &imageService{
		db:        db,
		imageRepo: imageRepo,
		cfg:       cfg,
	}
}

func (s *imageService) DeleteImage(imageID uuid.UUID) error {
	image, err := s.imageRepo.FindByID(imageID)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "Image", IDString: imageID.String()}
	}

	fileExtension := filepath.Ext(image.OriginalFilename)
	filename := image.ID.String() + fileExtension
	filePath := filepath.Join(s.cfg.ImageUploadPath, filename)

	if err := os.Remove(filePath); err != nil {
		// Log the error but don't fail if the file is already gone
		if !os.IsNotExist(err) {
			return apperrors.New(500, "failed to delete image file from disk", err)
		}
	}

	return s.imageRepo.Delete(imageID)
}

func (s *imageService) ListImages(folder string) ([]models.Image, error) {
	return s.imageRepo.ListByFolder(folder)
}

func (s *imageService) UploadImage(fileHeader *multipart.FileHeader, folder string) (*models.Image, error) {
	file, err := fileHeader.Open()
	if err != nil {
		return nil, apperrors.New(500, "failed to open file", err)
	}
	defer file.Close()

	// Calculate hash
	hash := sha256.New()
	if _, err := io.Copy(hash, file); err != nil {
		return nil, apperrors.New(500, "failed to calculate file hash", err)
	}
	hashString := fmt.Sprintf("%x", hash.Sum(nil))

	// Check for duplicates
	existingImage, err := s.imageRepo.FindByHash(hashString)
	if err == nil && existingImage != nil {
		return existingImage, nil // Return existing image if found
	}
	if err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return nil, apperrors.New(500, "failed to check for image duplicates", err)
	}

	// Reset file reader
	if _, err := file.Seek(0, 0); err != nil {
		return nil, apperrors.New(500, "failed to reset file reader", err)
	}

	// Create new image record
	newImage := &models.Image{
		OriginalFilename: fileHeader.Filename,
		Hash:             hashString,
		MimeType:         fileHeader.Header.Get("Content-Type"),
		FileSize:         fileHeader.Size,
		Folder:           folder,
	}

	// The BeforeCreate hook will generate the UUID
	if err := newImage.BeforeCreate(nil); err != nil {
		return nil, apperrors.New(500, "failed to generate UUID for image", err)
	}

	// Save file to disk with UUID as name
	fileExtension := filepath.Ext(fileHeader.Filename)
	newFilename := newImage.ID.String() + fileExtension
	filePath := filepath.Join(s.cfg.ImageUploadPath, newFilename)

	outFile, err := os.Create(filePath)
	if err != nil {
		return nil, apperrors.New(500, "failed to create file on disk", err)
	}
	defer outFile.Close()

	if _, err := io.Copy(outFile, file); err != nil {
		return nil, apperrors.New(500, "failed to save file to disk", err)
	}

	// Save image record to database
	if err := s.imageRepo.Create(newImage); err != nil {
		// Clean up saved file if DB record fails
		os.Remove(filePath)
		return nil, apperrors.New(500, "failed to save image record to database", err)
	}

	return newImage, nil
}

func (s *imageService) GetImageFilePath(imageID uuid.UUID) (string, error) {
	image, err := s.imageRepo.FindByID(imageID)
	if err != nil {
		return "", &apperrors.ErrNotFound{Resource: "Image", IDString: imageID.String()}
	}

	fileExtension := filepath.Ext(image.OriginalFilename)
	filename := image.ID.String() + fileExtension
	filePath := filepath.Join(s.cfg.ImageUploadPath, filename)

	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		return "", apperrors.New(404, "image file not found on disk", err)
	}

	return filePath, nil
}
