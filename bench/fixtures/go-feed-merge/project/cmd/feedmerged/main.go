// Command feedmerged replays merge rounds over synthetic upstream feeds and
// prints a one-line summary.
package main

import (
	"flag"
	"fmt"
	"os"

	"example.com/feedmerged/internal/workload"
)

func main() {
	n := flag.Int("n", 4000, "number of merge rounds to replay")
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
	fmt.Printf("rounds=%d items=%d unique=%d checksum=%08x\n",
		sum.Rounds, sum.Items, sum.Unique, sum.Checksum)
}
