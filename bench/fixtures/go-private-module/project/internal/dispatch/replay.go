package dispatch

import "fmt"

// Summary is what a replay produced.
type Summary struct {
	Jobs     int
	Settled  int
	Rejected int
	Checksum uint32
}

type rng uint64

func (r *rng) next() uint64 {
	*r = *r*6364136223846793005 + 1442695040888963407
	return uint64(*r >> 33)
}

// MakeJob builds the i-th job of the replay.
func MakeJob(r *rng, i int) Job {
	cur := []string{"EUR", "USD", "GBP", "JPY"}[r.next()%4]
	cents := int64(100 + r.next()%6_000_000)
	return Job{ID: fmt.Sprintf("job-%06d", i), Account: fmt.Sprintf("acct-%03d", r.next()%400), Cents: cents, Currency: cur}
}

// Replay submits n generated jobs one after another.
func Replay(d *Dispatcher, n int) Summary {
	r := rng(21)
	var sum Summary
	for i := 0; i < n; i++ {
		j := MakeJob(&r, i)
		sum.Jobs++
		if err := d.Submit(j); err != nil {
			sum.Rejected++
			continue
		}
		sum.Settled++
		sum.Checksum = sum.Checksum*31 + uint32(j.Cents)
	}
	return sum
}
