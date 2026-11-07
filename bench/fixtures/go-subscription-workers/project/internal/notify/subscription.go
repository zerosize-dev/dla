package notify

import "sync"

// Subscription delivers messages for one recipient from its own worker, so a
// slow sink only delays that recipient's messages.
type Subscription struct {
	hub       *Hub
	recipient string
	topic     string

	events  chan Message
	flushes chan chan struct{}
	done    chan struct{}
	exited  chan struct{}
	once    sync.Once

	scratch   []byte
	delivered int
}

func newSubscription(h *Hub, recipient, topic string) *Subscription {
	s := &Subscription{
		hub:       h,
		recipient: recipient,
		topic:     topic,
		events:    make(chan Message),
		flushes:   make(chan chan struct{}),
		done:      make(chan struct{}),
		exited:    make(chan struct{}),
		scratch:   make([]byte, 0, 4096),
	}
	go s.run()
	return s
}

func (s *Subscription) run() {
	defer close(s.exited)
	for {
		select {
		case m := <-s.events:
			s.deliver(m)
		case ack := <-s.flushes:
			close(ack)
		case <-s.done:
			return
		}
	}
}

func (s *Subscription) deliver(m Message) {
	s.scratch = s.hub.renderer.Render(s.scratch[:0], m)
	s.hub.sink.Deliver(s.recipient, s.scratch)
	s.delivered++
}

// Push hands a message to the worker. It returns once the worker has taken it.
func (s *Subscription) Push(m Message) error {
	m.Topic = s.topic
	select {
	case s.events <- m:
		return nil
	case <-s.done:
		return ErrClosed
	}
}

// Flush waits until every message pushed so far has been delivered.
func (s *Subscription) Flush() error {
	ack := make(chan struct{})
	select {
	case s.flushes <- ack:
		<-ack
		return nil
	case <-s.done:
		return ErrClosed
	}
}

// Delivered reports how many messages the worker has handled. Call it after
// Flush or Close.
func (s *Subscription) Delivered() int {
	return s.delivered
}

// Close stops the worker and waits for it to exit. It is safe to call twice.
func (s *Subscription) Close() {
	s.once.Do(func() { close(s.done) })
	<-s.exited
}
