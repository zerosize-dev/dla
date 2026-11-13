package reserve

import (
	"context"
	"time"
)

// awaitConfirm calls check until it reports true, once per interval, and
// gives up when ctx ends. It reports whether check succeeded.
func awaitConfirm(ctx context.Context, interval time.Duration, check func() bool) bool {
	tick := time.NewTicker(interval)
	if check() {
		return true
	}
	for {
		select {
		case <-tick.C:
			if check() {
				return true
			}
		case <-ctx.Done():
			return false
		}
	}
}
