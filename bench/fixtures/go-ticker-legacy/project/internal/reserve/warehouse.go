package reserve

import "fmt"

// Warehouse confirms holds. The simulated warehouse confirms a hold as soon
// as it is placed, apart from a hold on a SKU it has marked as slow, which
// it confirms on the third check.
type Warehouse struct {
	next   int
	slow   map[string]bool
	checks map[string]int
	sku    map[string]string
}

// NewWarehouse returns a warehouse; the SKUs in slow confirm late.
func NewWarehouse(slow ...string) *Warehouse {
	w := &Warehouse{slow: map[string]bool{}, checks: map[string]int{}, sku: map[string]string{}}
	for _, s := range slow {
		w.slow[s] = true
	}
	return w
}

// Hold places a hold for qty units of sku and returns its id.
func (w *Warehouse) Hold(sku string, qty int) string {
	w.next++
	id := fmt.Sprintf("H%07d", w.next)
	w.sku[id] = sku
	return id
}

// Confirmed reports whether the warehouse has confirmed the hold. A confirmed
// hold is forgotten.
func (w *Warehouse) Confirmed(id string) bool {
	w.checks[id]++
	if w.slow[w.sku[id]] && w.checks[id] < 3 {
		return false
	}
	delete(w.checks, id)
	delete(w.sku, id)
	return true
}
