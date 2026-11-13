package rates

import (
	"context"
	"time"

	"example.com/ratesd/internal/trace"
)

// Poller answers lookups from its cache and refreshes a pair from the source
// when the cached quote is too old.
type Poller struct {
	src   Source
	cache *Cache
	now   func() time.Time

	// ctx is the context of the poller's current round of work. Each lookup
	// starts its span from it, so every span can be traced back to the
	// lookups before it.
	ctx context.Context

	refreshes int
}

// NewPoller returns a poller over src with a cache of the given ttl.
func NewPoller(src Source, ttl time.Duration, now func() time.Time) *Poller {
	return &Poller{src: src, cache: NewCache(ttl), now: now, ctx: context.Background()}
}

// Lookup returns the rate of pair and whether it came from the cache.
func (p *Poller) Lookup(pair string) (Quote, bool, error) {
	p.ctx = trace.Start(p.ctx, "lookup", trace.Attr{Key: "pair", Value: pair})
	now := p.now()
	if q, ok := p.cache.Get(pair, now); ok {
		return q, true, nil
	}
	q, err := p.src.Fetch(p.ctx, pair)
	if err != nil {
		return Quote{}, false, err
	}
	p.cache.Put(q)
	p.refreshes++
	return q, false, nil
}

// Refreshes reports how many quotes were fetched from the source.
func (p *Poller) Refreshes() int {
	return p.refreshes
}

// Where describes the trace path of the poller's latest lookup.
func (p *Poller) Where() string {
	return trace.Path(p.ctx, 3)
}
