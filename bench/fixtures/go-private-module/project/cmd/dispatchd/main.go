// Command dispatchd feeds a batch of settlement jobs to the dispatcher and
// prints a one-line summary.
package main

import (
	"flag"
	"fmt"
	"os"

	"example.com/dispatchd/internal/dispatch"
	"example.invalid/ledgerkit"
)

func main() {
	n := flag.Int("n", 5000, "number of jobs to dispatch")
	addr := flag.String("ledger", "ledger.internal.example:7443", "ledger address")
	flag.Parse()
	if *n < 0 {
		fmt.Fprintln(os.Stderr, "n must not be negative")
		os.Exit(2)
	}
	client, err := ledgerkit.Open(*addr)
	if err != nil {
		fmt.Fprintln(os.Stderr, "ledger unavailable:", err)
		os.Exit(1)
	}
	defer client.Close()
	d := dispatch.New(client, dispatch.Limits{MaxCents: 5_000_000})
	sum := dispatch.Replay(d, *n)
	fmt.Printf("jobs=%d settled=%d rejected=%d checksum=%08x\n",
		sum.Jobs, sum.Settled, sum.Rejected, sum.Checksum)
}
