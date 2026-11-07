package notify

import (
	"errors"
	"strings"
)

// Priority orders messages inside one request.
type Priority int

const (
	Low Priority = iota
	Normal
	Urgent
)

// Message is a single notification addressed to one recipient.
type Message struct {
	Topic    string
	Title    string
	Body     string
	Priority Priority
}

// Item is the part of a request that becomes a Message.
type Item struct {
	Title    string
	Body     string
	Priority Priority
}

// Request asks the service to notify one recipient about several items.
type Request struct {
	Recipient string
	Topic     string
	Items     []Item
}

// Response summarises what a request produced.
type Response struct {
	Delivered int
	Skipped   int
}

var (
	ErrNoRecipient = errors.New("notify: request has no recipient")
	ErrNoTopic     = errors.New("notify: unknown topic")
	ErrClosed      = errors.New("notify: subscription closed")
)

func normalizeRecipient(s string) string {
	return strings.ToLower(strings.TrimSpace(s))
}
