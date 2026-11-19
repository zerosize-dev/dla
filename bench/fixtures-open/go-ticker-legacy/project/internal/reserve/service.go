package reserve

import (
	"context"
	"time"
)

// Order asks for units of one SKU.
type Order struct {
	ID  string
	SKU string
	Qty int
}

// Reservation is a confirmed hold.
type Reservation struct {
	OrderID string
	HoldID  string
	SKU     string
	Qty     int
}

// Service reserves stock for orders.
type Service struct {
	stock    *Stock
	wh       *Warehouse
	interval time.Duration
	timeout  time.Duration
}

// NewService returns a service that polls the warehouse every interval and
// waits at most timeout for a confirmation.
func NewService(stock *Stock, wh *Warehouse, interval, timeout time.Duration) *Service {
	return &Service{stock: stock, wh: wh, interval: interval, timeout: timeout}
}

// Reserve takes stock for the order and waits for the warehouse to confirm
// the hold. Stock is returned when the confirmation does not come.
func (s *Service) Reserve(ctx context.Context, o Order) (Reservation, error) {
	if err := s.stock.Take(o.SKU, o.Qty); err != nil {
		return Reservation{}, err
	}
	hold := s.wh.Hold(o.SKU, o.Qty)
	ctx, cancel := context.WithTimeout(ctx, s.timeout)
	defer cancel()
	if !awaitConfirm(ctx, s.interval, func() bool { return s.wh.Confirmed(hold) }) {
		s.stock.Put(o.SKU, o.Qty)
		return Reservation{}, context.DeadlineExceeded
	}
	return Reservation{OrderID: o.ID, HoldID: hold, SKU: o.SKU, Qty: o.Qty}, nil
}

// Release returns a reservation's units to stock.
func (s *Service) Release(r Reservation) {
	s.stock.Put(r.SKU, r.Qty)
}
