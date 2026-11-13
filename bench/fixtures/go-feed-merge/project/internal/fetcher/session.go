package fetcher

import "sync"

// Session is one read of an upstream feed. Its own goroutine owns the
// upstream call so a slow feed does not block the merge round that asked.
type Session struct {
	feed string
	seq  int
	up   Upstream

	reads chan chan []byte
	done  chan struct{}
	once  sync.Once
	exit  chan struct{}

	last []byte
}

func newSession(feed string, up Upstream, seq int) *Session {
	s := &Session{
		feed:  feed,
		seq:   seq,
		up:    up,
		reads: make(chan chan []byte),
		done:  make(chan struct{}),
		exit:  make(chan struct{}),
		last:  make([]byte, 0, 1024),
	}
	go s.serve()
	return s
}

func (s *Session) serve() {
	defer close(s.exit)
	for {
		select {
		case reply := <-s.reads:
			doc := s.up(s.seq)
			s.last = append(s.last[:0], doc[:min(len(doc), 64)]...)
			reply <- doc
		case <-s.done:
			return
		}
	}
}

// Read returns the document the upstream produced for this session.
func (s *Session) Read() []byte {
	reply := make(chan []byte)
	select {
	case s.reads <- reply:
		return <-reply
	case <-s.done:
		return nil
	}
}

// Feed returns the name of the feed the session reads.
func (s *Session) Feed() string {
	return s.feed
}

// Close ends the session and waits for its goroutine to exit. It is safe to
// call twice.
func (s *Session) Close() {
	s.once.Do(func() { close(s.done) })
	<-s.exit
}
