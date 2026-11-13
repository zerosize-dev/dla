package tally

import "fmt"

// Report is the one-line outcome of a batch.
type Report struct {
	Batch    int
	Applied  int
	Rejected int
	Net      int64
}

// Recent keeps the last few reports for the status page.
type Recent struct {
	limit int
	items []Report
}

// NewRecent returns a list that keeps the last limit reports.
func NewRecent(limit int) *Recent {
	return &Recent{limit: limit}
}

// Add appends a report and drops the oldest ones beyond the limit.
func (r *Recent) Add(rep Report) {
	r.items = append(r.items, rep)
	if len(r.items) > r.limit {
		r.items = r.items[len(r.items)-r.limit:]
	}
}

// Len reports how many reports are kept.
func (r *Recent) Len() int {
	return len(r.items)
}

// Lines renders the kept reports, oldest first.
func (r *Recent) Lines() []string {
	out := make([]string, 0, len(r.items))
	for _, it := range r.items {
		out = append(out, fmt.Sprintf("batch %d: applied=%d rejected=%d net=%d", it.Batch, it.Applied, it.Rejected, it.Net))
	}
	return out
}

// Pipeline decodes batches and applies them to a ledger.
type Pipeline struct {
	ledger *Ledger
	recent *Recent
	batch  int
}

// NewPipeline returns a pipeline over ledger that keeps recent reports.
func NewPipeline(l *Ledger, recent *Recent) *Pipeline {
	return &Pipeline{ledger: l, recent: recent}
}

// Process decodes one batch, applies its events and records the report.
func (p *Pipeline) Process(batch []byte) Report {
	p.batch++
	events, rejected := Decode(batch)
	rep := Report{Batch: p.batch, Rejected: rejected}
	for _, ev := range events {
		if !p.ledger.Apply(ev) {
			rep.Rejected++
			continue
		}
		rep.Applied++
		if ev.Kind == Debit {
			rep.Net -= ev.Cents
		} else {
			rep.Net += ev.Cents
		}
	}
	p.recent.Add(rep)
	return rep
}
