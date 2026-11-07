package p

import (
	"database/sql"
	"runtime"
	"sync"
	"time"
)

type T struct{ f *int }

var pl sync.Pool

func Put() { pl.Put([4]byte{}) }

func Tk() <-chan time.Time { return time.Tick(time.Second) }

func Fin2() {
	t := &T{}
	runtime.SetFinalizer(t, func(x *T) { _ = t.f })
}

func Ch(c chan int) {
	for range c {
		defer println("x")
	}
}

func Q2(db *sql.DB) error {
	rows, err := db.Query("select 1")
	if err != nil {
		return err
	}
	for rows.Next() {
	}
	return nil
}
