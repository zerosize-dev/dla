package rates

import "time"

// Cache holds the latest quote of every pair.
type Cache struct {
	ttl   time.Duration
	quote map[string]Quote
}

// NewCache returns a cache whose entries expire after ttl.
func NewCache(ttl time.Duration) *Cache {
	return &Cache{ttl: ttl, quote: make(map[string]Quote)}
}

// Get returns the quote for pair when it is younger than the ttl at now.
func (c *Cache) Get(pair string, now time.Time) (Quote, bool) {
	q, ok := c.quote[pair]
	if !ok || now.Sub(q.At) > c.ttl {
		return Quote{}, false
	}
	return q, true
}

// Put stores q as the latest quote of its pair.
func (c *Cache) Put(q Quote) {
	c.quote[q.Pair] = q
}

// Len reports how many pairs have a quote.
func (c *Cache) Len() int {
	return len(c.quote)
}
