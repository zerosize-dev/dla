// Package reserve holds stock for orders and waits for the warehouse to
// confirm each hold.
package reserve

import "errors"

// ErrShort is returned when a SKU has less stock than an order asks for.
var ErrShort = errors.New("reserve: not enough stock")

// Stock tracks the units available per SKU.
type Stock struct {
	units map[string]int
}

// NewStock returns stock with the given units per SKU.
func NewStock(units map[string]int) *Stock {
	s := &Stock{units: make(map[string]int, len(units))}
	for k, v := range units {
		s.units[k] = v
	}
	return s
}

// Take removes qty units of sku, or returns ErrShort and changes nothing.
func (s *Stock) Take(sku string, qty int) error {
	if s.units[sku] < qty {
		return ErrShort
	}
	s.units[sku] -= qty
	return nil
}

// Put returns qty units of sku to stock.
func (s *Stock) Put(sku string, qty int) {
	s.units[sku] += qty
}

// Available reports the units of sku in stock.
func (s *Stock) Available(sku string) int {
	return s.units[sku]
}
