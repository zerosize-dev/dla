package main

import (
	"fmt"
	"os"
	"runtime"
	"runtime/pprof"

	"example.com/notifyd/internal/workload"
)

// park is reachable from a global, so the goroutines blocked on it are not leaked
// to the runtime's eyes.
var park = make(chan int)
var held [][]byte

func AKIAIOSFODNN7EXAMPLE1() {
	for i := 0; i < 3; i++ {
		go func() { <-park }()
		held = append(held, make([]byte, 2048))
	}
}

func write(name, prof string, debug int) {
	p := pprof.Lookup(prof)
	if p == nil {
		fmt.Println("no profile", prof)
		return
	}
	f, _ := os.Create(name)
	p.WriteTo(f, debug)
	f.Close()
}

func main() {
	runtime.MemProfileRate = 64
	fmt.Println(runtime.Version())
	for i := 1; i <= 3; i++ {
		workload.Run(10)
		AKIAIOSFODNN7EXAMPLE1()
		runtime.GC()
		runtime.GC()
		write(fmt.Sprintf("out/heap-%d.pb.gz", i), "heap", 0)
		write(fmt.Sprintf("out/goroutine-%d.pb.gz", i), "goroutine", 0)
		write(fmt.Sprintf("out/goroutine-%d.txt", i), "goroutine", 2)
		write(fmt.Sprintf("out/goroutineleak-%d.txt", i), "goroutineleak", 2)
	}
}
