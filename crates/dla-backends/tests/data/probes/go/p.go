package p

import (
	"context"
	"database/sql"
	"net/http"
	"sync"
	"time"
	"runtime"
	"os"
)

type S struct{ ctx context.Context }

var pool = sync.Pool{New: func() any { return [4]byte{} }}

func Get(u string) {
	resp, _ := http.Get(u)
	_ = resp
}

func Q(db *sql.DB) {
	rows, _ := db.Query("select 1")
	for rows.Next() {
	}
}

func Tick() {
	for range time.Tick(time.Second) {
	}
}

func Cancel(ctx context.Context) {
	ctx, cancel := context.WithCancel(ctx)
	_ = ctx
	_ = cancel
}

func Fat(ctx context.Context) {
	for i := 0; i < 3; i++ {
		ctx, _ = context.WithCancel(ctx)
	}
}

func Loops(paths []string) {
	for _, p := range paths {
		f, _ := os.Open(p)
		defer f.Close()
	}
	for {
		defer println("x")
	}
}

func Fin() {
	x := new(int)
	runtime.SetFinalizer(x, func(y *int) { _ = x })
}

func Lock(m *sync.Mutex) {
	m.Lock()
	m.Unlock()
	m.Lock()
	defer m.Lock()
}
