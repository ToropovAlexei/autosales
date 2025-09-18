package services

import (
	"frbktg/backend_go/repositories"
	"time"

	"golang.org/x/sync/errgroup"
)

type DashboardStats struct {
	TotalUsers         int64 `json:"total_users"`
	UsersWithPurchases int64 `json:"users_with_purchases"`
	AvailableProducts  int64 `json:"available_products"`
}

type SalesOverTime struct {
	ProductsSold int64   `json:"products_sold"`
	TotalRevenue float64 `json:"total_revenue"`
}

type DashboardService interface {
	GetDashboardStats() (*DashboardStats, error)
	GetSalesOverTime(start, end time.Time) (*SalesOverTime, error)
}

type dashboardService struct {
	dashboardRepo repositories.DashboardRepository
}

func NewDashboardService(dashboardRepo repositories.DashboardRepository) DashboardService {
	return &dashboardService{dashboardRepo: dashboardRepo}
}

func (s *dashboardService) GetDashboardStats() (*DashboardStats, error) {
	var g errgroup.Group

	stats := &DashboardStats{}

	g.Go(func() error {
		var err error
		stats.TotalUsers, err = s.dashboardRepo.CountTotalUsers()
		return err
	})

	g.Go(func() error {
		var err error
		stats.UsersWithPurchases, err = s.dashboardRepo.CountUsersWithPurchases()
		return err
	})

	g.Go(func() error {
		var err error
		stats.AvailableProducts, err = s.dashboardRepo.CountAvailableProducts()
		return err
	})

	if err := g.Wait(); err != nil {
		return nil, err
	}

	return stats, nil
}

func (s *dashboardService) GetSalesOverTime(start, end time.Time) (*SalesOverTime, error) {
	var g errgroup.Group

	salesData := &SalesOverTime{}

	g.Go(func() error {
		var err error
		salesData.ProductsSold, err = s.dashboardRepo.GetSalesCountForPeriod(start, end)
		return err
	})

	g.Go(func() error {
		var err error
		salesData.TotalRevenue, err = s.dashboardRepo.GetTotalRevenueForPeriod(start, end)
		return err
	})

	if err := g.Wait(); err != nil {
		return nil, err
	}

	return salesData, nil
}
