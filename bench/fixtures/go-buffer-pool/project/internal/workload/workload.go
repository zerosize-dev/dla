// Package workload generates deterministic invoice batches.
package workload

import (
	"fmt"

	"example.com/invoiced/internal/render"
)

// Summary is what a replay produced.
type Summary struct {
	Batches  int
	Invoices int
	Bytes    int
	Digest   uint64
}

const maxBatch = 400

type rng uint64

func (r *rng) next() uint64 {
	*r = *r*6364136223846793005 + 1442695040888963407
	return uint64(*r >> 33)
}

var skus = []string{"HW-100", "HW-210", "SV-007", "SV-031", "LC-900", "TR-512"}
var descs = []string{"rack unit", "support hour", "licence seat", "travel day", "spare part", "setup fee"}

// NewRNG returns the generator that Run uses.
func NewRNG() *rng {
	r := rng(7)
	return &r
}

// MakeBatch builds the i-th batch of the replay.
func MakeBatch(r *rng, i int) []render.Invoice {
	size := 1 + int(r.next()%maxBatch)
	batch := make([]render.Invoice, 0, size)
	for k := 0; k < size; k++ {
		inv := render.Invoice{
			ID:       fmt.Sprintf("INV-%06d-%03d", i, k),
			Customer: fmt.Sprintf("customer-%d", r.next()%5000),
			Currency: "EUR",
		}
		lines := 1 + int(r.next()%80)
		for j := 0; j < lines; j++ {
			s := int(r.next() % uint64(len(skus)))
			inv.Lines = append(inv.Lines, render.Line{
				SKU:       skus[s],
				Desc:      descs[s],
				Quantity:  1 + int(r.next()%20),
				UnitCents: int64(100 + r.next()%90000),
			})
		}
		batch = append(batch, inv)
	}
	return batch
}

// Run replays n batches through a fresh renderer.
func Run(n int) (Summary, error) {
	rd := render.NewRenderer(render.NewBufferPool())
	r := NewRNG()
	var sum Summary
	for i := 0; i < n; i++ {
		res := rd.RenderBatch(MakeBatch(r, i))
		sum.Batches++
		sum.Invoices += res.Invoices
		sum.Bytes += res.Bytes
		sum.Digest ^= res.Digest
	}
	return sum, nil
}

// NewRenderer returns the renderer and pool that Run builds, for harnesses.
func NewRenderer() (*render.Renderer, *render.BufferPool) {
	p := render.NewBufferPool()
	return render.NewRenderer(p), p
}
