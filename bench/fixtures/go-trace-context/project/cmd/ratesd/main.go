// Command ratesd replays synthetic rate lookups through the quote poller
// and prints a one-line summary.
package main

import (
	"flag"
	"fmt"
	"os"

	"example.com/ratesd/internal/workload"
)

func main() {
	n := flag.Int("n", 100000, "number of lookups to replay")
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
	fmt.Printf("lookups=%d fresh=%d cached=%d checksum=%08x\n",
		sum.Lookups, sum.Fresh, sum.Cached, sum.Checksum)
}
