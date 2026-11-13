// Package tally decodes batches of account events and keeps a daily ledger
// per account.
package tally

import (
	"bytes"
	"errors"
	"strconv"
)

// Kind says what an event does to an account.
type Kind byte

// The event kinds.
const (
	Credit Kind = 'C'
	Debit  Kind = 'D'
)

// Event is one decoded line of a batch.
type Event struct {
	Account string
	Day     int
	Cents   int64
	Kind    Kind
}

// ErrBadLine is returned for a line that is not "account,day,cents,kind".
var ErrBadLine = errors.New("tally: malformed event line")

// Decode parses a batch. Lines that do not parse are counted as rejected.
func Decode(batch []byte) (events []Event, rejected int) {
	for len(batch) > 0 {
		line := batch
		if i := bytes.IndexByte(batch, '\n'); i >= 0 {
			line, batch = batch[:i], batch[i+1:]
		} else {
			batch = nil
		}
		ev, err := parseLine(line)
		if err != nil {
			rejected++
			continue
		}
		events = append(events, ev)
	}
	return events, rejected
}

func parseLine(line []byte) (Event, error) {
	f := bytes.Split(line, []byte(","))
	if len(f) != 4 || len(f[3]) != 1 {
		return Event{}, ErrBadLine
	}
	day, err := strconv.Atoi(string(f[1]))
	if err != nil {
		return Event{}, ErrBadLine
	}
	cents, err := strconv.ParseInt(string(f[2]), 10, 64)
	if err != nil {
		return Event{}, ErrBadLine
	}
	k := Kind(f[3][0])
	if k != Credit && k != Debit {
		return Event{}, ErrBadLine
	}
	return Event{Account: string(f[0]), Day: day, Cents: cents, Kind: k}, nil
}
