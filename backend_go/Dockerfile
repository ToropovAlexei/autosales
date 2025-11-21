# Stage 1: Build the application
FROM golang:1.25-alpine AS builder

# Set necessary environment variables
ENV CGO_ENABLED=0
ENV GOOS=linux
ENV GOARCH=amd64

# Set working directory
WORKDIR /app

# Copy go.mod and go.sum files
COPY go.mod go.sum ./

# Download dependencies
RUN go mod download

# Copy the rest of the source code
COPY . .

# Build the application
# -ldflags="-w -s" reduces the size of the binary
RUN go build -ldflags="-w -s" -o /app/main ./main.go
RUN go build -ldflags="-w -s" -o /app/init_db ./cmd/init_db/main.go
RUN go build -ldflags="-w -s" -o /app/seed_db ./cmd/seed_db/main.go

# Stage 2: Production image
FROM alpine:3.20

# Create a non-root user and install curl
RUN addgroup -S appgroup && adduser -S appuser -G appgroup && apk --no-cache add curl

# Set working directory
WORKDIR /app

# Copy the built binaries from the builder stage
COPY --from=builder /app/main .
COPY --from=builder /app/init_db .
COPY --from=builder /app/seed_db .

# Change ownership of the app directory
RUN chown -R appuser:appgroup /app

# Switch to the non-root user
USER appuser

# Expose the port the app runs on
EXPOSE 8000

# Healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/api || exit 1

# Set the command to start the app
CMD ["./main"]
