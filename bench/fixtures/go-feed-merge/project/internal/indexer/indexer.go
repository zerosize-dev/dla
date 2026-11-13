// Package indexer keeps the most recent headlines for the search box's
// suggestions.
package indexer

import "bytes"

// Indexer remembers the newest headlines in a ring of fixed size.
type Indexer struct {
	ring  [][]byte
	next  int
	count int
}

// New returns an indexer that keeps the last size headlines.
func New(size int) *Indexer {
	return &Indexer{ring: make([][]byte, size)}
}

// Remember adds a headline, replacing the oldest one when the ring is full.
func (x *Indexer) Remember(headline []byte) {
	if len(headline) == 0 {
		return
	}
	x.ring[x.next] = headline
	x.next = (x.next + 1) % len(x.ring)
	if x.count < len(x.ring) {
		x.count++
	}
}

// Len reports how many headlines are remembered.
func (x *Indexer) Len() int {
	return x.count
}

// Suggest returns up to limit remembered headlines that contain prefix,
// newest first.
func (x *Indexer) Suggest(prefix []byte, limit int) [][]byte {
	var out [][]byte
	for i := 1; i <= x.count && len(out) < limit; i++ {
		h := x.ring[(x.next-i+len(x.ring))%len(x.ring)]
		if bytes.Contains(bytes.ToLower(h), bytes.ToLower(prefix)) {
			out = append(out, h)
		}
	}
	return out
}
