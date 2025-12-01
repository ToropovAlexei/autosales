package services

type BotStatusService interface {
	GetBotStatus() (bool, error)
}

type botStatusService struct {
	storeBalanceService StoreBalanceService
}

func NewBotStatusService(storeBalanceService StoreBalanceService) BotStatusService {
	return &botStatusService{storeBalanceService: storeBalanceService}
}

func (s *botStatusService) GetBotStatus() (bool, error) {
	balance, err := s.storeBalanceService.GetStoreBalance()
	if err != nil {
		// In case of an error (e.g., DB connection issue), default to not operating.
		return false, err
	}

	// If balance is less than 1, bot cannot operate.
	return balance.CurrentBalance >= 1.0, nil
}
