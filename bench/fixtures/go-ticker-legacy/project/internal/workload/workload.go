// Package workload generates deterministic order traffic for the
// reservation service.
package workload

import (
	"context"
	"errors"
	"fmt"
	"time"

	"example.com/reservd/internal/reserve"
)

// Summary is what a replay produced.
type Summary struct {
	Orders   int
	Reserved int
	Short    int
	Checksum uint32
}

var skus = []string{"bolt-m6", "bolt-m8", "nut-m6", "washer-6", "bracket-a", "bracket-b", "plate-12", "hinge-s"}

type rng uint64

func (r *rng) next() uint64 {
	*r = *r*6364136223846793005 + 1442695040888963407
	return uint64(*r >> 33)
}

// NewService builds the service used by Run: a 30-second poll interval and a
// two-minute timeout, over a warehouse that confirms immediately.
func NewService() *reserve.Service {
	units := map[string]int{}
	for _, s := range skus {
		units[s] = 500
	}
	return reserve.NewService(reserve.NewStock(units), reserve.NewWarehouse(), 30*time.Second, 2*time.Minute)
}

// MakeOrder builds the i-th order of the replay.
func MakeOrder(r *rng, i int) reserve.Order {
	return reserve.Order{
		ID:  fmt.Sprintf("ord-%07d", i),
		SKU: skus[r.next()%uint64(len(skus))],
		Qty: 1 + int(r.next()%40),
	}
}

// Run replays n orders. Every reservation is released after a short delay
// in order, so stock turns over.
func Run(n int) (Summary, error) {
	svc := NewService()
	r := rng(13)
	var sum Summary
	var held []reserve.Reservation
	for i := 0; i < n; i++ {
		res, err := svc.Reserve(context.Background(), MakeOrder(&r, i))
		sum.Orders++
		switch {
		case errors.Is(err, reserve.ErrShort):
			sum.Short++
		case err != nil:
			return sum, fmt.Errorf("order %d: %w", i, err)
		default:
			sum.Reserved++
			sum.Checksum = sum.Checksum*31 + uint32(res.Qty)
			held = append(held, res)
		}
		if len(held) > 25 {
			svc.Release(held[0])
			held = held[1:]
		}
	}
	return sum, nil
}
