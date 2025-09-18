package services

import (
	"errors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"
)

type BalanceService interface {
	DepositBalance(userID int64, amount float64, description string) error
}

type balanceService struct {
	balanceRepo repositories.BalanceRepository
	botUserRepo repositories.BotUserRepository
}

func NewBalanceService(balanceRepo repositories.BalanceRepository, botUserRepo repositories.BotUserRepository) BalanceService {
	return &balanceService{balanceRepo: balanceRepo, botUserRepo: botUserRepo}
}

func (s *balanceService) DepositBalance(userID int64, amount float64, description string) error {
	user, err := s.botUserRepo.FindByTelegramID(userID)
	if err != nil {
		return errors.New("bot user not found")
	}

	transaction := &models.Transaction{
		UserID:      user.ID,
		Type:        models.Deposit,
		Amount:      amount,
		Description: description,
		CreatedAt:   time.Now().UTC(),
	}

	return s.balanceRepo.CreateDepositTransaction(transaction)
}
