// Command tallyd replays batches of account events through the tally
// pipeline and prints a one-line summary.
package main

import (
	"flag"
	"fmt"
	"os"

	"example.com/tallyd/internal/workload"
)

func main() {
	n := flag.Int("n", 200, "number of batches to replay")
	flag.Parse()
	if *n < 0 {
		fmt.Fprintln(os.Stderr, "n must not be negative")
		os.Exit(2)
	}
	sum, err := workload.Run(*n)
	if err != nil {
		fmt.Fprintln(os.Stderr, "replay failed:", err)
		os.Exit(1)
	}
	fmt.Printf("batches=%d events=%d rejected=%d checksum=%08x\n",
		sum.Batches, sum.Events, sum.Rejected, sum.Checksum)
}
