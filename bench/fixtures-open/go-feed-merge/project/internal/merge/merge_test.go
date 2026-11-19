package merge

import (
	"testing"

	"example.com/feedmerged/internal/fetcher"
	"example.com/feedmerged/internal/indexer"
)

func TestParseSkipsBadLines(t *testing.T) {
	doc := []byte("a|10|first\nbroken\nb|x|bad time\nc|30|third|with bar\n")
	items := Parse("f", doc)
	if len(items) != 2 || items[0].ID != "a" || items[1].Time != 30 {
		t.Fatalf("unexpected items: %+v", items)
	}
	if string(items[1].Headline) != "third|with bar" || items[1].Feed != "f" {
		t.Fatalf("unexpected headline: %q", items[1].Headline)
	}
}

func upstreams() map[string]fetcher.Upstream {
	return map[string]fetcher.Upstream{
		"x": func(seq int) []byte { return []byte("x1|5|early\nx2|9|late\n") },
		"y": func(seq int) []byte { return []byte("y1|7|middle\nx1|5|early\n") },
	}
}

func TestRoundMergesNewestFirst(t *testing.T) {
	idx := indexer.New(10)
	m := NewMerger(fetcher.New(upstreams()), []string{"x", "y"}, idx, 2)
	res, err := m.Round()
	if err != nil {
		t.Fatal(err)
	}
	if res.Items != 4 || res.Unique != 3 {
		t.Fatalf("want 4 items and 3 unique, got %d and %d", res.Items, res.Unique)
	}
	if len(res.Top) != 2 || string(res.Top[0].Headline) != "late" || string(res.Top[1].Headline) != "middle" {
		t.Fatalf("unexpected top: %+v", res.Top)
	}
	if idx.Len() != 2 {
		t.Fatalf("top headlines should be indexed, got %d", idx.Len())
	}
}

func TestRoundDropsItemsSeenBefore(t *testing.T) {
	m := NewMerger(fetcher.New(upstreams()), []string{"x"}, indexer.New(10), 5)
	m.Round()
	res, _ := m.Round()
	if res.Unique != 0 {
		t.Fatalf("second round should see nothing new, got %d", res.Unique)
	}
}

func TestRoundUnknownFeed(t *testing.T) {
	m := NewMerger(fetcher.New(upstreams()), []string{"nope"}, indexer.New(10), 5)
	if _, err := m.Round(); err == nil {
		t.Fatal("an unknown feed should fail the round")
	}
}
