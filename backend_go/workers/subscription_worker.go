package workers

import (
	"frbktg/backend_go/repositories"
	"frbktg/backend_go/services"
	"log/slog"
	"time"

	"github.com/go-co-op/gocron"
)

type SubscriptionWorker struct {
	scheduler            *gocron.Scheduler
	orderService         services.OrderService
	userSubscriptionRepo repositories.UserSubscriptionRepository
	logger               *slog.Logger
}

func NewSubscriptionWorker(orderService services.OrderService, userSubscriptionRepo repositories.UserSubscriptionRepository, logger *slog.Logger) *SubscriptionWorker {
	return &SubscriptionWorker{
		scheduler:            gocron.NewScheduler(time.UTC),
		orderService:         orderService,
		userSubscriptionRepo: userSubscriptionRepo,
		logger:               logger,
	}
}

func (w *SubscriptionWorker) Start() {
	w.scheduler.Every(24).Hours().Do(w.renewSubscriptions)
	w.scheduler.StartAsync()
	w.logger.Info("Subscription renewal worker started")
}

func (w *SubscriptionWorker) renewSubscriptions() {
	w.logger.Info("Starting subscription renewal job")

	subscriptions, err := w.userSubscriptionRepo.GetExpiringSubscriptions(24 * time.Hour)
	if err != nil {
		w.logger.Error("failed to get expiring subscriptions", "error", err)
		return
	}

	w.logger.Info("Found subscriptions to renew", "count", len(subscriptions))

	for _, sub := range subscriptions {
		if err := w.orderService.RenewSubscription(sub.ID); err != nil {
			w.logger.Error("failed to renew subscription", "subscription_id", sub.ID, "error", err)
		}
	}

	w.logger.Info("Subscription renewal job finished")
}
