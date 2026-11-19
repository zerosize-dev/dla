package indexer

import "testing"

func TestRingKeepsTheNewest(t *testing.T) {
	x := New(3)
	for _, h := range []string{"one", "two", "three", "four"} {
		x.Remember([]byte(h))
	}
	if x.Len() != 3 {
		t.Fatalf("want 3 headlines, got %d", x.Len())
	}
	got := x.Suggest([]byte(""), 10)
	if len(got) != 3 || string(got[0]) != "four" || string(got[2]) != "two" {
		t.Fatalf("unexpected order: %q", got)
	}
}

func TestSuggestMatchesCaseInsensitively(t *testing.T) {
	x := New(5)
	x.Remember([]byte("Rates Fall"))
	x.Remember([]byte("Storm warning"))
	got := x.Suggest([]byte("rates"), 5)
	if len(got) != 1 || string(got[0]) != "Rates Fall" {
		t.Fatalf("unexpected suggestions: %q", got)
	}
	if len(x.Suggest([]byte(""), 1)) != 1 {
		t.Fatal("limit should apply")
	}
}

func TestRememberIgnoresEmpty(t *testing.T) {
	x := New(2)
	x.Remember(nil)
	if x.Len() != 0 {
		t.Fatal("an empty headline should not be stored")
	}
}
