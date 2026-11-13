// Package rates polls an upstream rate source and answers lookups from the
// latest quotes.
package rates

import "time"

// Quote is one currency pair's rate.
type Quote struct {
	Pair string
	Bid  float64
	Ask  float64
	At   time.Time
	Span uint64 // id of the span that fetched the quote, 0 when untraced
}

// Mid returns the middle of bid and ask.
func (q Quote) Mid() float64 {
	return (q.Bid + q.Ask) / 2
}
