// Package workload generates deterministic lookup traffic for the quote
// poller.
package workload

import (
	"fmt"
	"time"

	"example.com/ratesd/internal/rates"
)

// Summary is what a replay produced.
type Summary struct {
	Lookups  int
	Fresh    int
	Cached   int
	Checksum uint32
}

// Pairs are the currency pairs the replay asks about.
func Pairs() []string {
	return []string{"EUR/USD", "GBP/USD", "USD/JPY", "USD/CHF", "AUD/USD", "USD/CAD", "NZD/USD", "EUR/GBP"}
}

type rng uint64

func (r *rng) next() uint64 {
	*r = *r*6364136223846793005 + 1442695040888963407
	return uint64(*r >> 33)
}

// NewPoller builds the poller used by Run, on a clock the caller advances.
func NewPoller(clock *time.Time) *rates.Poller {
	now := func() time.Time { return *clock }
	src := &rates.SimulatedSource{Pairs: Pairs(), Clock: now}
	return rates.NewPoller(src, 2*time.Second, now)
}

// Run replays n lookups, advancing a simulated clock by 40 ms per lookup.
func Run(n int) (Summary, error) {
	clock := time.Date(2025, 3, 1, 9, 0, 0, 0, time.UTC)
	p := NewPoller(&clock)
	pairs := Pairs()
	r := rng(11)
	var sum Summary
	for i := 0; i < n; i++ {
		pair := pairs[r.next()%uint64(len(pairs))]
		q, cached, err := p.Lookup(pair)
		if err != nil {
			return sum, fmt.Errorf("lookup %d: %w", i, err)
		}
		sum.Lookups++
		if cached {
			sum.Cached++
		} else {
			sum.Fresh++
		}
		sum.Checksum = sum.Checksum*31 + uint32(q.Mid()*10000)
		clock = clock.Add(40 * time.Millisecond)
	}
	return sum, nil
}
