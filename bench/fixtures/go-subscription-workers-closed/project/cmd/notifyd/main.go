// Command notifyd replays synthetic notification traffic through the
// delivery pipeline and prints a one-line summary.
package main

import (
	"flag"
	"fmt"
	"os"

	"example.com/notifyd/internal/workload"
)

func main() {
	n := flag.Int("n", 2000, "number of requests to replay")
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
	fmt.Printf("requests=%d delivered=%d bytes=%d checksum=%08x\n",
		sum.Requests, sum.Delivered, sum.Bytes, sum.Checksum)
}
