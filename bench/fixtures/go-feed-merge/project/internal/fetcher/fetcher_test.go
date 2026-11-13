package fetcher

import (
	"errors"
	"testing"
)

func echo(seq int) []byte { return []byte{byte('a' + seq%26)} }

func TestOpenUnknownFeed(t *testing.T) {
	f := New(map[string]Upstream{"a": echo})
	if _, err := f.Open("b"); !errors.Is(err, ErrUnknownFeed) {
		t.Fatalf("want ErrUnknownFeed, got %v", err)
	}
}

func TestSessionReadsItsDocument(t *testing.T) {
	f := New(map[string]Upstream{"a": echo})
	s1, _ := f.Open("a")
	s2, _ := f.Open("a")
	defer s1.Close()
	defer s2.Close()
	if string(s1.Read()) != "b" || string(s2.Read()) != "c" {
		t.Fatal("each session should read the document of its own sequence number")
	}
	if s1.Feed() != "a" {
		t.Fatalf("unexpected feed %q", s1.Feed())
	}
}

func TestCloseTwiceAndReadAfterClose(t *testing.T) {
	f := New(map[string]Upstream{"a": echo})
	s, _ := f.Open("a")
	s.Close()
	s.Close()
	if s.Read() != nil {
		t.Fatal("a closed session returns no document")
	}
}
