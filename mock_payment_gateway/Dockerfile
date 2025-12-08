# Stage 1: Build
FROM golang:1.25-alpine AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN go build -ldflags="-w -s" -o /app/main .

# Stage 2: Run
FROM alpine:3.20

WORKDIR /app

RUN addgroup -S appgroup && adduser -S appuser -G appgroup

COPY --from=builder /app/main .
COPY --from=builder /app/payment.html .

RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 8078

CMD ["./main"]
