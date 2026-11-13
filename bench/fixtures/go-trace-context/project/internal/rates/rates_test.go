package rates

import (
	"context"
	"errors"
	"testing"
	"time"

	"example.com/ratesd/internal/trace"
)

func fixedClock(t *time.Time) func() time.Time {
	return func() time.Time { return *t }
}

func TestSimulatedSourceQuotes(t *testing.T) {
	now := time.Unix(1000, 0)
	s := &SimulatedSource{Pairs: []string{"EUR/USD"}, Clock: fixedClock(&now)}
	q, err := s.Fetch(context.Background(), "EUR/USD")
	if err != nil || q.Ask <= q.Bid || q.Pair != "EUR/USD" {
		t.Fatalf("bad quote %+v, %v", q, err)
	}
	if _, err := s.Fetch(context.Background(), "XXX/YYY"); !errors.Is(err, ErrUnknownPair) {
		t.Fatalf("want ErrUnknownPair, got %v", err)
	}
}

func TestSourceTagsTheSpan(t *testing.T) {
	now := time.Unix(1000, 0)
	s := &SimulatedSource{Pairs: []string{"EUR/USD"}, Clock: fixedClock(&now)}
	q, _ := s.Fetch(context.Background(), "EUR/USD")
	if q.Span != 0 {
		t.Fatalf("untraced fetch should carry no span, got %d", q.Span)
	}
	ctx := trace.Start(context.Background(), "x")
	q, _ = s.Fetch(ctx, "EUR/USD")
	if q.Span != trace.From(ctx).ID {
		t.Fatalf("quote should carry the span id, got %d", q.Span)
	}
}

func TestCacheExpires(t *testing.T) {
	c := NewCache(time.Second)
	at := time.Unix(1000, 0)
	c.Put(Quote{Pair: "A", Bid: 1, Ask: 2, At: at})
	if _, ok := c.Get("A", at.Add(500*time.Millisecond)); !ok {
		t.Fatal("young entry should be served")
	}
	if _, ok := c.Get("A", at.Add(2*time.Second)); ok {
		t.Fatal("old entry should not be served")
	}
}

func TestPollerRefreshesWhenOld(t *testing.T) {
	now := time.Unix(1000, 0)
	src := &SimulatedSource{Pairs: []string{"EUR/USD"}, Clock: fixedClock(&now)}
	p := NewPoller(src, time.Second, fixedClock(&now))
	if _, cached, _ := p.Lookup("EUR/USD"); cached {
		t.Fatal("first lookup cannot be cached")
	}
	if _, cached, _ := p.Lookup("EUR/USD"); !cached {
		t.Fatal("second lookup should be cached")
	}
	now = now.Add(3 * time.Second)
	if _, cached, _ := p.Lookup("EUR/USD"); cached {
		t.Fatal("lookup after the ttl should refresh")
	}
	if p.Refreshes() != 2 {
		t.Fatalf("want 2 refreshes, got %d", p.Refreshes())
	}
}

func TestPollerReportsUnknownPair(t *testing.T) {
	now := time.Unix(1000, 0)
	p := NewPoller(&SimulatedSource{Clock: fixedClock(&now)}, time.Second, fixedClock(&now))
	if _, _, err := p.Lookup("A/B"); !errors.Is(err, ErrUnknownPair) {
		t.Fatalf("want ErrUnknownPair, got %v", err)
	}
}

func TestWhereNamesTheLookup(t *testing.T) {
	now := time.Unix(1000, 0)
	p := NewPoller(&SimulatedSource{Pairs: []string{"A/B"}, Clock: fixedClock(&now)}, time.Second, fixedClock(&now))
	p.Lookup("A/B")
	if p.Where() != "lookup" {
		t.Fatalf("unexpected path %q", p.Where())
	}
}
