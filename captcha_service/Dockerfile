# --- Build Stage ---
FROM golang:1.25-alpine AS builder

# Set the Current Working Directory inside the container
WORKDIR /app

# Copy go mod and sum files
COPY go.mod go.sum ./

# Download all dependencies. Dependencies will be cached if the go.mod and go.sum files are not changed
RUN go mod download

# Copy the source code
COPY . .

# Build the Go app
# -ldflags="-w -s" strips debug information, resulting in a smaller binary
# CGO_ENABLED=0 disables CGO for a statically linked binary
RUN CGO_ENABLED=0 GOOS=linux go build -a -ldflags="-w -s" -o captcha_service .

# --- Final Stage ---
FROM alpine:latest

WORKDIR /root/

# Copy the Pre-built binary file from the previous stage
COPY --from=builder /app/captcha_service .

# Expose port 9091 to the outside world
EXPOSE 9091

# Command to run the executable
CMD ["./captcha_service"]
