package rates

import (
	"context"
	"errors"
	"hash/fnv"
	"time"

	"example.com/ratesd/internal/trace"
)

// ErrUnknownPair is returned for a pair the source does not quote.
var ErrUnknownPair = errors.New("rates: unknown pair")

// Source returns the current quote for a pair.
type Source interface {
	Fetch(ctx context.Context, pair string) (Quote, error)
}

// SimulatedSource derives quotes from the pair name and a counter, with no
// network access.
type SimulatedSource struct {
	Pairs []string
	Clock func() time.Time
	calls int
}

// Fetch returns a deterministic quote that drifts with every call.
func (s *SimulatedSource) Fetch(ctx context.Context, pair string) (Quote, error) {
	known := false
	for _, p := range s.Pairs {
		if p == pair {
			known = true
			break
		}
	}
	if !known {
		return Quote{}, ErrUnknownPair
	}
	s.calls++
	h := fnv.New32a()
	h.Write([]byte(pair))
	base := 0.5 + float64(h.Sum32()%2000)/1000
	drift := float64(s.calls%97) / 10000
	q := Quote{Pair: pair, Bid: base + drift, Ask: base + drift + 0.0004, At: s.Clock()}
	if sp := trace.From(ctx); sp != nil {
		q.Span = sp.ID
	}
	return q, nil
}
