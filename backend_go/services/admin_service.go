package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type AdminService interface {
	GetBotUsersWithBalance(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.BotUserResponse], error)
}

type adminService struct {
	adminRepo   repositories.AdminRepository
	botUserRepo repositories.BotUserRepository
}

func NewAdminService(adminRepo repositories.AdminRepository, botUserRepo repositories.BotUserRepository) AdminService {
	return &adminService{adminRepo: adminRepo, botUserRepo: botUserRepo}
}

func (s *adminService) GetBotUsersWithBalance(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.BotUserResponse], error) {
	paginatedUsers, err := s.adminRepo.GetActiveBotUsers(page, filters)
	if err != nil {
		return nil, err
	}

	var response []models.BotUserResponse
	for _, u := range paginatedUsers.Data {
		balance, err := s.botUserRepo.GetUserBalance(u.ID)
		if err != nil {
			// Depending on requirements, you might want to log this error but continue
			return nil, err
		}

		response = append(response, models.BotUserResponse{
			ID:                u.ID,
			TelegramID:        u.TelegramID,
			IsBlocked:         u.IsBlocked,
			HasPassedCaptcha:  u.HasPassedCaptcha,
			Balance:           balance,
			RegisteredWithBot: u.RegisteredWithBot,
			LastSeenWithBot:   u.LastSeenWithBot,
			CreatedAt:         u.CreatedAt,
			LastSeenAt:        u.LastSeenAt,
		})
	}

	return &models.PaginatedResult[models.BotUserResponse]{
		Data:  response,
		Total: paginatedUsers.Total,
	}, nil
}
