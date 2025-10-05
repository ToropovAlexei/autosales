package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type AdminService interface {
	GetBotUsersWithBalance() ([]models.BotUserResponse, error)
	SoftDeleteBotUser(id uint) error
}

type adminService struct {
	adminRepo   repositories.AdminRepository
	botUserRepo repositories.BotUserRepository
}

func NewAdminService(adminRepo repositories.AdminRepository, botUserRepo repositories.BotUserRepository) AdminService {
	return &adminService{adminRepo: adminRepo, botUserRepo: botUserRepo}
}

func (s *adminService) GetBotUsersWithBalance() ([]models.BotUserResponse, error) {
	botUsers, err := s.adminRepo.GetActiveBotUsers()
	if err != nil {
		return nil, err
	}

	var response []models.BotUserResponse
	for _, u := range botUsers {
		balance, err := s.botUserRepo.GetUserBalance(u.ID)
		if err != nil {
			// Depending on requirements, you might want to log this error but continue
			return nil, err
		}

		response = append(response, models.BotUserResponse{
			ID:                u.ID,
			TelegramID:        u.TelegramID,
			IsDeleted:         u.IsDeleted,
			HasPassedCaptcha:  u.HasPassedCaptcha,
			Balance:           balance,
			RegisteredWithBot: u.RegisteredWithBot,
			LastSeenWithBot:   u.LastSeenWithBot,
			CreatedAt:         u.CreatedAt,
		})
	}

	return response, nil
}

func (s *adminService) SoftDeleteBotUser(id uint) error {
	user, err := s.adminRepo.GetBotUserByID(id)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "BotUser", ID: id}
	}
	return s.adminRepo.SoftDeleteBotUser(user)
}
