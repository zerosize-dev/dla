package notify

import (
	"hash/fnv"
	"sync"
	"sync/atomic"
)

// Hub opens subscriptions that render and deliver messages for a recipient.
type Hub struct {
	renderer *Renderer
	sink     Sink
	opened   atomic.Int64
}

// NewHub wires a renderer and a sink together.
func NewHub(r *Renderer, s Sink) *Hub {
	return &Hub{renderer: r, sink: s}
}

// Open starts a subscription for the recipient on a topic.
func (h *Hub) Open(recipient, topic string) (*Subscription, error) {
	recipient = normalizeRecipient(recipient)
	if recipient == "" {
		return nil, ErrNoRecipient
	}
	if !h.renderer.Knows(topic) {
		return nil, ErrNoTopic
	}
	h.opened.Add(1)
	return newSubscription(h, recipient, topic), nil
}

// Opened returns how many subscriptions this hub has created.
func (h *Hub) Opened() int64 {
	return h.opened.Load()
}

// Sink receives rendered notifications.
type Sink interface {
	Deliver(recipient string, text []byte)
}

// MemorySink counts what it receives and keeps a running checksum, which is
// enough for replay runs and tests.
type MemorySink struct {
	mu       sync.Mutex
	messages int
	bytes    int
	sum      uint32
}

// Deliver records one notification.
func (s *MemorySink) Deliver(recipient string, text []byte) {
	h := fnv.New32a()
	h.Write([]byte(recipient))
	h.Write(text)
	s.mu.Lock()
	s.messages++
	s.bytes += len(text)
	s.sum ^= h.Sum32()
	s.mu.Unlock()
}

// Totals returns the message count, byte count and checksum so far.
func (s *MemorySink) Totals() (messages, bytes int, sum uint32) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.messages, s.bytes, s.sum
}
