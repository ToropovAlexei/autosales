package services

import (
	"context"
	"encoding/json"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"log/slog"

	"github.com/go-redis/redis/v8"
)

type BroadcastService interface {
	GetFilteredUsers(filters models.BroadcastFilters, page models.Page) (*models.PaginatedResult[models.BotUser], error)
	StartBroadcast(payload models.BroadcastPayload) error
}

type broadcastService struct {
	botUserRepo repositories.BotUserRepository
	redisClient *redis.Client
}

func NewBroadcastService(botUserRepo repositories.BotUserRepository, redisClient *redis.Client) BroadcastService {
	return &broadcastService{
		botUserRepo: botUserRepo,
		redisClient: redisClient,
	}
}

func (s *broadcastService) GetFilteredUsers(filters models.BroadcastFilters, page models.Page) (*models.PaginatedResult[models.BotUser], error) {
	return s.botUserRepo.GetBotUsersForBroadcast(filters, page)
}

func (s *broadcastService) StartBroadcast(payload models.BroadcastPayload) error {
	if (payload.Text == nil || *payload.Text == "") && (payload.ImageID == nil || *payload.ImageID == "") {
		return apperrors.New(400, "Broadcast message must contain at least text or an image_id", nil)
	}

	go func() {
		users, err := s.botUserRepo.GetAllBotUsersForBroadcast(payload.Filters)
		if err != nil {
			slog.Error("failed to get users for broadcast", "error", err)
			return
		}

		slog.Info("starting broadcast", "user_count", len(users))

		for _, user := range users {
			redisPayload := models.RedisBroadcastMessage{
				TelegramID: user.TelegramID,
				Text:       payload.Text,
				ImageID:    payload.ImageID,
			}

			payloadBytes, err := json.Marshal(redisPayload)
			if err != nil {
				slog.Error("failed to marshal broadcast payload for user", "error", err, "user_id", user.ID)
				continue
			}

			channel := "bot-notifications:" + user.RegisteredWithBot
			if err := s.redisClient.Publish(context.Background(), channel, payloadBytes).Err(); err != nil {
				slog.Error("failed to publish broadcast message to redis", "error", err, "user_id", user.ID, "channel", channel)
			}
		}

		slog.Info("broadcast finished", "user_count", len(users))
	}()

	return nil
}
