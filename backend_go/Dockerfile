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

# Stage 2: Production image
FROM alpine:latest

# Set working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/main .

# Copy the .env file
# Note: For production, it's better to use environment variables
# passed to the container instead of packaging the .env file.
COPY .env.example .env

# Expose the port the app runs on
EXPOSE 8000

# Set the command to start the app
CMD ["./main"]
