// Package workload generates deterministic request traffic for the
// notification pipeline.
package workload

import (
	"fmt"

	"example.com/notifyd/internal/notify"
)

// Summary is what a replay produced.
type Summary struct {
	Requests  int
	Delivered int
	Bytes     int
	Checksum  uint32
}

var topics = []string{"billing", "shipping", "security", "digest"}

// Layouts returns the renderer layouts used by the replay.
func Layouts() map[string]string {
	return map[string]string{
		"billing":  "[billing] {title} - {body}",
		"shipping": "[shipping] {title}: {body}",
		"security": "!! {title} !! {body}",
		"digest":   "{title}\n{body}",
	}
}

type rng uint64

func (r *rng) next() uint64 {
	*r = *r*6364136223846793005 + 1442695040888963407
	return uint64(*r >> 33)
}

// Build creates the pipeline used by Run and returns it with its sink.
func Build() (*notify.Handler, *notify.MemorySink) {
	sink := &notify.MemorySink{}
	hub := notify.NewHub(notify.NewRenderer(Layouts()), sink)
	return notify.NewHandler(hub, 6), sink
}

// Run replays n requests, one at a time.
func Run(n int) (Summary, error) {
	h, sink := Build()
	r := rng(42)
	var sum Summary
	for i := 0; i < n; i++ {
		resp, err := h.Serve(MakeRequest(&r, i))
		if err != nil {
			return sum, fmt.Errorf("request %d: %w", i, err)
		}
		sum.Requests++
		sum.Delivered += resp.Delivered
	}
	_, sum.Bytes, sum.Checksum = sink.Totals()
	return sum, nil
}

// MakeRequest builds the i-th request of the replay.
func MakeRequest(r *rng, i int) notify.Request {
	req := notify.Request{
		Recipient: fmt.Sprintf("User-%d@example.test", r.next()%977),
		Topic:     topics[r.next()%uint64(len(topics))],
	}
	count := 1 + int(r.next()%5)
	for k := 0; k < count; k++ {
		req.Items = append(req.Items, notify.Item{
			Title:    fmt.Sprintf("event %d.%d", i, k),
			Body:     fmt.Sprintf("payload %x", r.next()),
			Priority: notify.Priority(r.next() % 3),
		})
	}
	return req
}

// NewRNG returns the generator used by Run, for harnesses that drive the
// handler themselves.
func NewRNG() *rng {
	r := rng(42)
	return &r
}
