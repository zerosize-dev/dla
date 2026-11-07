// Command invoiced replays synthetic invoice batches through the renderer
// and prints a one-line summary.
package main

import (
	"flag"
	"fmt"
	"os"

	"example.com/invoiced/internal/workload"
)

func main() {
	n := flag.Int("n", 2000, "number of batches to replay")
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
	fmt.Printf("batches=%d invoices=%d bytes=%d digest=%016x\n",
		sum.Batches, sum.Invoices, sum.Bytes, sum.Digest)
}
