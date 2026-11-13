package reserve

import (
	"context"
	"errors"
	"testing"
	"time"
)

func TestStockTakePut(t *testing.T) {
	s := NewStock(map[string]int{"a": 5})
	if err := s.Take("a", 3); err != nil || s.Available("a") != 2 {
		t.Fatalf("take failed: %v, %d left", err, s.Available("a"))
	}
	if err := s.Take("a", 3); !errors.Is(err, ErrShort) || s.Available("a") != 2 {
		t.Fatalf("a short take should change nothing: %v", err)
	}
	s.Put("a", 4)
	if s.Available("a") != 6 {
		t.Fatalf("want 6, got %d", s.Available("a"))
	}
}

func TestWarehouseConfirmsLateForSlowSKUs(t *testing.T) {
	w := NewWarehouse("slow")
	fast, slow := w.Hold("fast", 1), w.Hold("slow", 1)
	if !w.Confirmed(fast) {
		t.Fatal("a fast hold confirms on the first check")
	}
	if w.Confirmed(slow) || w.Confirmed(slow) || !w.Confirmed(slow) {
		t.Fatal("a slow hold confirms on the third check")
	}
}

func TestAwaitConfirmPollsUntilTrue(t *testing.T) {
	calls := 0
	ok := awaitConfirm(context.Background(), time.Millisecond, func() bool {
		calls++
		return calls == 3
	})
	if !ok || calls != 3 {
		t.Fatalf("want success on the third check, got %v after %d", ok, calls)
	}
}

func TestAwaitConfirmStopsWhenContextEnds(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 20*time.Millisecond)
	defer cancel()
	if awaitConfirm(ctx, time.Millisecond, func() bool { return false }) {
		t.Fatal("a check that never succeeds should time out")
	}
}

func TestReserveHoldsAndReleases(t *testing.T) {
	stock := NewStock(map[string]int{"a": 10})
	svc := NewService(stock, NewWarehouse("a"), time.Millisecond, time.Second)
	res, err := svc.Reserve(context.Background(), Order{ID: "o1", SKU: "a", Qty: 4})
	if err != nil || res.Qty != 4 || stock.Available("a") != 6 {
		t.Fatalf("reserve failed: %+v %v, %d left", res, err, stock.Available("a"))
	}
	svc.Release(res)
	if stock.Available("a") != 10 {
		t.Fatalf("release should return the units, %d in stock", stock.Available("a"))
	}
}

func TestReserveGivesStockBackOnTimeout(t *testing.T) {
	stock := NewStock(map[string]int{"a": 10})
	svc := NewService(stock, NewWarehouse("a"), time.Hour, 20*time.Millisecond)
	if _, err := svc.Reserve(context.Background(), Order{ID: "o1", SKU: "a", Qty: 4}); !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("want deadline exceeded, got %v", err)
	}
	if stock.Available("a") != 10 {
		t.Fatalf("stock should be back at 10, got %d", stock.Available("a"))
	}
}
