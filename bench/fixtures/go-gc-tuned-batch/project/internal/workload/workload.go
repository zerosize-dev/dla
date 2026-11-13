// Package workload generates deterministic event batches for the tally
// pipeline.
package workload

import (
	"bytes"
	"fmt"

	"example.com/tallyd/internal/tally"
)

// Summary is what a replay produced.
type Summary struct {
	Batches  int
	Events   int
	Rejected int
	Checksum uint32
}

// Accounts is the number of accounts the replay opens.
const Accounts = 3000

// EventsPerBatch is the number of lines in each batch.
const EventsPerBatch = 60000

type rng uint64

func (r *rng) next() uint64 {
	*r = *r*6364136223846793005 + 1442695040888963407
	return uint64(*r >> 33)
}

// NewPipeline returns the pipeline used by Run, with every account open.
func NewPipeline() *tally.Pipeline {
	l := tally.NewLedger()
	for i := 0; i < Accounts; i++ {
		l.Open(fmt.Sprintf("acct-%04d", i))
	}
	return tally.NewPipeline(l, tally.NewRecent(32))
}

// MakeBatch builds the i-th batch of the replay.
func MakeBatch(r *rng, i int) []byte {
	var b bytes.Buffer
	for k := 0; k < EventsPerBatch; k++ {
		kind := "C"
		if r.next()%3 == 0 {
			kind = "D"
		}
		if r.next()%997 == 0 {
			fmt.Fprintf(&b, "garbled line %d\n", k)
			continue
		}
		fmt.Fprintf(&b, "acct-%04d,%d,%d,%s\n", r.next()%(Accounts+20), r.next()%tally.DaysKept, 100+r.next()%9900, kind)
	}
	return b.Bytes()
}

// Run replays n batches.
func Run(n int) (Summary, error) {
	p := NewPipeline()
	r := rng(5)
	var sum Summary
	for i := 0; i < n; i++ {
		rep := p.Process(MakeBatch(&r, i))
		sum.Batches++
		sum.Events += rep.Applied
		sum.Rejected += rep.Rejected
		sum.Checksum = sum.Checksum*31 + uint32(rep.Net) + uint32(rep.Applied)
	}
	return sum, nil
}
