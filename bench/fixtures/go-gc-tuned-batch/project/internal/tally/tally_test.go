package tally

import "testing"

func TestDecodeSkipsBadLines(t *testing.T) {
	batch := []byte("a,1,500,C\nnonsense\na,2,x,D\nb,3,250,D\na,4,10,Z\n")
	events, rejected := Decode(batch)
	if len(events) != 2 || rejected != 3 {
		t.Fatalf("want 2 events and 3 rejected, got %d and %d", len(events), rejected)
	}
	if events[1].Account != "b" || events[1].Day != 3 || events[1].Kind != Debit {
		t.Fatalf("unexpected event %+v", events[1])
	}
}

func TestLedgerApply(t *testing.T) {
	l := NewLedger()
	l.Open("a")
	if !l.Apply(Event{Account: "a", Day: 2, Cents: 700, Kind: Credit}) ||
		!l.Apply(Event{Account: "a", Day: 3, Cents: 200, Kind: Debit}) {
		t.Fatal("events for an open account should apply")
	}
	if l.Balance("a") != 500 {
		t.Fatalf("want balance 500, got %d", l.Balance("a"))
	}
	if l.Apply(Event{Account: "b", Day: 1, Cents: 1, Kind: Credit}) {
		t.Fatal("unknown account should be refused")
	}
	if l.Apply(Event{Account: "a", Day: DaysKept, Cents: 1, Kind: Credit}) {
		t.Fatal("a day out of range should be refused")
	}
	if l.Accounts() != 1 {
		t.Fatalf("want 1 account, got %d", l.Accounts())
	}
}

func TestRecentKeepsTheLast(t *testing.T) {
	r := NewRecent(2)
	for i := 1; i <= 5; i++ {
		r.Add(Report{Batch: i})
	}
	lines := r.Lines()
	if r.Len() != 2 || len(lines) != 2 || lines[0][:7] != "batch 4" {
		t.Fatalf("unexpected lines %q", lines)
	}
}

func TestPipelineProcess(t *testing.T) {
	l := NewLedger()
	l.Open("a")
	p := NewPipeline(l, NewRecent(4))
	rep := p.Process([]byte("a,1,100,C\na,2,30,D\nzz,1,5,C\nbroken\n"))
	if rep.Applied != 2 || rep.Rejected != 2 || rep.Net != 70 || rep.Batch != 1 {
		t.Fatalf("unexpected report %+v", rep)
	}
}
