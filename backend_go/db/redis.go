package db

import (
	"context"
	"frbktg/backend_go/config"

	"github.com/go-redis/redis/v8"
)

func InitRedis(cfg *config.Config) *redis.Client {
	rdb := redis.NewClient(&redis.Options{
		Addr: cfg.RedisAddr,
	})

	// Ping the server to check the connection
	_, err := rdb.Ping(context.Background()).Result()
	if err != nil {
		panic("failed to connect to redis: " + err.Error())
	}

	return rdb
}
