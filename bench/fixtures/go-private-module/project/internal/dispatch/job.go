// Package dispatch sends settlement jobs to the ledger and records how each
// one ended.
package dispatch

import (
	"errors"
	"fmt"
)

// Job is one settlement to post.
type Job struct {
	ID       string
	Account  string
	Cents    int64
	Currency string
}

// Limits are the checks a job must pass before it is sent.
type Limits struct {
	MaxCents int64
}

// ErrRejected is returned for a job that fails its checks.
var ErrRejected = errors.New("dispatch: job rejected")

// Check validates a job against the limits.
func (l Limits) Check(j Job) error {
	switch {
	case j.ID == "" || j.Account == "":
		return fmt.Errorf("%w: missing id or account", ErrRejected)
	case j.Cents <= 0 || j.Cents > l.MaxCents:
		return fmt.Errorf("%w: amount %d out of range", ErrRejected, j.Cents)
	case len(j.Currency) != 3:
		return fmt.Errorf("%w: currency %q", ErrRejected, j.Currency)
	}
	return nil
}
