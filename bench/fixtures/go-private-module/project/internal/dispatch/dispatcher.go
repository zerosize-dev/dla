package dispatch

import (
	"context"
	"sync"

	"example.invalid/ledgerkit"
)

// Outcome says how a job ended.
type Outcome struct {
	JobID   string
	Settled bool
	Detail  string
}

// Dispatcher posts jobs to the ledger and reports their outcomes.
type Dispatcher struct {
	client   *ledgerkit.Client
	limits   Limits
	ctx      context.Context
	mu       sync.Mutex
	outcomes map[string]Outcome
}

// New returns a dispatcher that posts through client.
func New(client *ledgerkit.Client, limits Limits) *Dispatcher {
	return &Dispatcher{
		client:   client,
		limits:   limits,
		ctx:      context.Background(),
		outcomes: make(map[string]Outcome),
	}
}

// Submit checks a job, posts it and starts a watcher for its receipt. It
// returns before the ledger has answered.
func (d *Dispatcher) Submit(j Job) error {
	if err := d.limits.Check(j); err != nil {
		return err
	}
	receipt, err := d.client.Post(ledgerkit.Entry{Key: j.ID, Account: j.Account, Cents: j.Cents, Currency: j.Currency})
	if err != nil {
		return err
	}
	go d.watch(j.ID, receipt)
	return nil
}

func (d *Dispatcher) watch(id string, receipt *ledgerkit.Receipt) {
	select {
	case res := <-receipt.Done():
		d.record(Outcome{JobID: id, Settled: res.OK, Detail: res.Message})
	case <-d.ctx.Done():
		return
	}
}

func (d *Dispatcher) record(o Outcome) {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.outcomes[o.JobID] = o
}

// Outcome returns what is known about a job.
func (d *Dispatcher) Outcome(id string) (Outcome, bool) {
	d.mu.Lock()
	defer d.mu.Unlock()
	o, ok := d.outcomes[id]
	return o, ok
}

// Pending reports how many jobs have no outcome yet among the ids given.
func (d *Dispatcher) Pending(ids []string) int {
	d.mu.Lock()
	defer d.mu.Unlock()
	n := 0
	for _, id := range ids {
		if _, ok := d.outcomes[id]; !ok {
			n++
		}
	}
	return n
}
