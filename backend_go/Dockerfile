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
FROM alpine:latest

# Set working directory
WORKDIR /app

# Copy the built binaries from the builder stage
COPY --from=builder /app/main .
COPY --from=builder /app/init_db .
COPY --from=builder /app/seed_db .

# Copy the .env file
# Note: For production, it's better to use environment variables
# passed to the container instead of packaging the .env file.
COPY .env.example .env

# Expose the port the app runs on
EXPOSE 8000

# Set the command to start the app
CMD ["./main"]
