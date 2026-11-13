// Package fetcher reads documents from upstream feeds.
package fetcher

import (
	"errors"
	"fmt"
)

// ErrUnknownFeed is returned for a feed the fetcher has no upstream for.
var ErrUnknownFeed = errors.New("fetcher: unknown feed")

// Upstream produces the next document of a feed.
type Upstream func(seq int) []byte

// Fetcher opens sessions on its upstream feeds.
type Fetcher struct {
	feeds map[string]Upstream
	seq   map[string]int
}

// New returns a fetcher over the given upstream feeds.
func New(feeds map[string]Upstream) *Fetcher {
	return &Fetcher{feeds: feeds, seq: make(map[string]int)}
}

// Open starts a session on feed.
func (f *Fetcher) Open(feed string) (*Session, error) {
	up, ok := f.feeds[feed]
	if !ok {
		return nil, fmt.Errorf("%w: %q", ErrUnknownFeed, feed)
	}
	f.seq[feed]++
	return newSession(feed, up, f.seq[feed]), nil
}
