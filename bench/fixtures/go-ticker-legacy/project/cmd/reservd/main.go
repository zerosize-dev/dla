// Command reservd replays synthetic orders through the stock reservation
// service and prints a one-line summary.
package main

import (
	"flag"
	"fmt"
	"os"

	"example.com/reservd/internal/workload"
)

func main() {
	n := flag.Int("n", 200000, "number of orders to replay")
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
	fmt.Printf("orders=%d reserved=%d short=%d checksum=%08x\n",
		sum.Orders, sum.Reserved, sum.Short, sum.Checksum)
}
