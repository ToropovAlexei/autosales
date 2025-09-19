package apperrors

import "fmt"

// ErrNotFound represents a resource not found error.
// To be converted to a 404 HTTP status.
type ErrNotFound struct {
	Resource string
	ID       interface{}
}

func (e *ErrNotFound) Error() string {
	return fmt.Sprintf("%s with ID %v not found", e.Resource, e.ID)
}

// ErrValidation represents a validation error.
// To be converted to a 400 HTTP status.
type ErrValidation struct {
	Message string
}

func (e *ErrValidation) Error() string {
	return e.Message
}

// ErrInsufficientBalance represents an insufficient balance error.
// To be converted to a 400 or 402 HTTP status.
type ErrInsufficientBalance struct {
}

func (e *ErrInsufficientBalance) Error() string {
	return "insufficient balance"
}

// ErrOutOfStock represents an out of stock error.
// To be converted to a 400 HTTP status.
type ErrOutOfStock struct {
	ProductName string
}

func (e *ErrOutOfStock) Error() string {
	return fmt.Sprintf("product %s is out of stock", e.ProductName)
}

// ErrAlreadyExists represents a resource that already exists.
// To be converted to a 409 HTTP status.
type ErrAlreadyExists struct {
	Resource string
	Field    string
	Value    string
}

func (e *ErrAlreadyExists) Error() string {
	return fmt.Sprintf("%s with %s '%s' already exists", e.Resource, e.Field, e.Value)
}

// ErrForbidden represents a forbidden action error.
// To be converted to a 403 HTTP status.
type ErrForbidden struct {
	Message string
}

func (e *ErrForbidden) Error() string {
	if e.Message == "" {
		return "user does not have enough privileges for this action"
	}
	return e.Message
}
