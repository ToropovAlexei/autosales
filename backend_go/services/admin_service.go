package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type AdminService interface {
	GetBotUsersWithBalance(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.BotUser], error)
}

type adminService struct {
	adminRepo   repositories.AdminRepository
	botUserRepo repositories.BotUserRepository
}

func NewAdminService(adminRepo repositories.AdminRepository, botUserRepo repositories.BotUserRepository) AdminService {
	return &adminService{adminRepo: adminRepo, botUserRepo: botUserRepo}
}

func (s *adminService) GetBotUsersWithBalance(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.BotUser], error) {
	return s.adminRepo.GetActiveBotUsers(page, filters)
}
