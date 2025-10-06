package workers

import (
	"frbktg/backend_go/services"
	"log/slog"
	"time"

	"github.com/go-co-op/gocron"
)

type PaymentWorker struct {
	scheduler      *gocron.Scheduler
	paymentService services.PaymentService
	logger         *slog.Logger
}

func NewPaymentWorker(paymentService services.PaymentService, logger *slog.Logger) *PaymentWorker {
	return &PaymentWorker{
		scheduler:      gocron.NewScheduler(time.UTC),
		paymentService: paymentService,
		logger:         logger,
	}
}

func (w *PaymentWorker) Start() {
	w.scheduler.Every(1).Minute().Do(w.checkUnfinishedPayments)
	w.scheduler.StartAsync()
	w.logger.Info("Payment worker started")
}

func (w *PaymentWorker) checkUnfinishedPayments() {
	w.logger.Info("Starting unfinished payments check job")

	if err := w.paymentService.NotifyUnfinishedPayments(); err != nil {
		w.logger.Error("failed to process unfinished payments", "error", err)
	}

	w.logger.Info("Unfinished payments check job finished")
}
