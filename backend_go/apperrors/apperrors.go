package apperrors

import (
	"fmt"
)

// Error represents a base error with a code and a message.
type Error struct {
	Code    int
	Message string
	Err     error
}

func (e *Error) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("code=%d, msg=%s, err=%v", e.Code, e.Message, e.Err)
	}
	return fmt.Sprintf("code=%d, msg=%s", e.Code, e.Message)
}

func New(code int, message string, err error) *Error {
	return &Error{Code: code, Message: message, Err: err}
}

// ErrNotFound is returned when a resource is not found.
type ErrNotFound struct {
	Base     *Error
	Resource string
	ID       uint
}

func (e *ErrNotFound) Error() string {
	return fmt.Sprintf("%s with ID %d not found", e.Resource, e.ID)
}

// ErrValidation is returned when input validation fails.
type ErrValidation struct {
	Base    *Error
	Message string
}

func (e *ErrValidation) Error() string {
	return e.Message
}

// ErrOutOfStock is returned when a product is out of stock.
type ErrOutOfStock struct {
	Base        *Error
	ProductName string
}

func (e *ErrOutOfStock) Error() string {
	return fmt.Sprintf("Product %s is out of stock", e.ProductName)
}

// ErrAlreadyExists is returned when a resource already exists.
type ErrAlreadyExists struct {
	Base     *Error
	Resource string
	Field    string
	Value    string
}

func (e *ErrAlreadyExists) Error() string {
	return fmt.Sprintf("%s with %s %s already exists", e.Resource, e.Field, e.Value)
}

var (
	ErrInsufficientBalance = &Error{Code: 402, Message: "Insufficient Balance"}
	ErrBotLimitExceeded    = &Error{Code: 400, Message: "Bot limit reached. A user can have a maximum of 3 bots."}
	ErrUnauthorized        = &Error{Code: 401, Message: "Unauthorized"}
	ErrForbidden           = &Error{Code: 403, Message: "Forbidden"}
	ErrInternalServer      = &Error{Code: 500, Message: "Internal Server Error"}
)
