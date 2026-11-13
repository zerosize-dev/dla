package dispatch

import (
	"errors"
	"testing"
)

func TestLimitsCheck(t *testing.T) {
	l := Limits{MaxCents: 1000}
	good := Job{ID: "a", Account: "x", Cents: 500, Currency: "EUR"}
	if err := l.Check(good); err != nil {
		t.Fatalf("good job rejected: %v", err)
	}
	for _, bad := range []Job{
		{Account: "x", Cents: 5, Currency: "EUR"},
		{ID: "a", Account: "x", Cents: 0, Currency: "EUR"},
		{ID: "a", Account: "x", Cents: 5000, Currency: "EUR"},
		{ID: "a", Account: "x", Cents: 5, Currency: "EURO"},
	} {
		if err := l.Check(bad); !errors.Is(err, ErrRejected) {
			t.Errorf("%+v: want ErrRejected, got %v", bad, err)
		}
	}
}

func TestMakeJobIsDeterministic(t *testing.T) {
	a, b := rng(21), rng(21)
	if MakeJob(&a, 3) != MakeJob(&b, 3) {
		t.Fatal("the same seed must give the same job")
	}
	r := rng(21)
	j := MakeJob(&r, 7)
	if j.ID != "job-000007" || len(j.Currency) != 3 {
		t.Fatalf("unexpected job %+v", j)
	}
}
