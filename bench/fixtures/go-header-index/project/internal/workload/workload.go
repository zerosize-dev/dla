// Package workload generates deterministic client traffic for the edge
// proxy.
package workload

import (
	"fmt"

	"example.com/edged/internal/edge"
)

// Summary is what a replay produced.
type Summary struct {
	Requests int
	Routed   int
	Denied   int
	Checksum uint32
}

// Rules is the routing table used by the replay.
func Rules() (map[string]string, []string) {
	return map[string]string{
			"/api/orders": "orders",
			"/api/users":  "accounts",
			"/static/":    "assets",
		},
		[]string{"/api/orders", "/api/users", "/static/"}
}

type rng uint64

func (r *rng) next() uint64 {
	*r = *r*6364136223846793005 + 1442695040888963407
	return uint64(*r >> 33)
}

var methods = []string{"GET", "GET", "POST", "PUT", "TRACE"}
var targets = []string{"/api/orders/17", "/api/users/9/profile", "/static/app.js", "/health", "/api/../etc"}

// Conn is a synthetic client connection: every Read fills p with the next
// request, padded with body bytes to the length of p.
type Conn struct {
	r rng
	i int
}

// NewConn returns a connection that starts at the first request.
func NewConn() *Conn {
	return &Conn{r: rng(7)}
}

// Read writes one request into p and returns len(p).
func (c *Conn) Read(p []byte) (int, error) {
	b := fmt.Appendf(p[:0], "%s %s HTTP/1.1\r\nHost: edge.example.test\r\nX-Tenant: tenant-%d\r\n",
		methods[c.r.next()%uint64(len(methods))], targets[c.r.next()%uint64(len(targets))], c.r.next()%53)
	b = fmt.Appendf(b, "X-Request-Id: req-%08x-%d\r\nAccept: */*\r\n\r\n", c.r.next(), c.i)
	for len(b)+16 <= len(p) {
		b = fmt.Appendf(b, "%016x", c.r.next())
	}
	for len(b) < len(p) {
		b = append(b, ' ')
	}
	c.i++
	return len(p), nil
}

// Run replays n frames through a router.
func Run(n int) (Summary, error) {
	rules, order := Rules()
	router := edge.NewRouter(rules, order, 100000)
	conn := NewConn()
	var sum Summary
	for i := 0; i < n; i++ {
		f, err := edge.ReadFrame(conn)
		if err != nil {
			return sum, err
		}
		route, err := router.Handle(f)
		if err != nil {
			return sum, fmt.Errorf("frame %d: %w", i, err)
		}
		sum.Requests++
		if route.Allowed {
			sum.Routed++
		} else {
			sum.Denied++
		}
		sum.Checksum = sum.Checksum*31 + uint32(len(route.Backend)) + uint32(edge.Shard([]byte(route.Backend), 8))
	}
	return sum, nil
}
