package edge

import (
	"bytes"
	"errors"
	"strings"
	"testing"
)

func frameOf(head string) *Frame {
	buf := make([]byte, FrameSize)
	n := copy(buf, head)
	return &Frame{buf: buf, n: n}
}

func TestParseRequest(t *testing.T) {
	f := frameOf("GET /api/orders/5 HTTP/1.1\r\nHost: x\r\nX-Tenant: t-4\r\nX-Request-Id: r-9\r\n\r\nbody")
	req, err := ParseRequest(f)
	if err != nil {
		t.Fatal(err)
	}
	if string(req.Method) != "GET" || string(req.Target) != "/api/orders/5" {
		t.Fatalf("bad request line: %q %q", req.Method, req.Target)
	}
	if string(req.Tenant) != "t-4" || string(req.Request) != "r-9" {
		t.Fatalf("bad headers: %q %q", req.Tenant, req.Request)
	}
}

func TestParseRequestShort(t *testing.T) {
	if _, err := ParseRequest(frameOf("GET")); !errors.Is(err, ErrShortFrame) {
		t.Fatalf("want ErrShortFrame, got %v", err)
	}
}

func TestReadFrameFillsBuffer(t *testing.T) {
	f, err := ReadFrame(strings.NewReader("GET / HTTP/1.1\r\n\r\n"))
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.HasPrefix(f.Bytes(), []byte("GET /")) || len(f.Bytes()) != 18 {
		t.Fatalf("unexpected frame: %q", f.Bytes())
	}
}

func TestIndexKeepsTheNewest(t *testing.T) {
	x := NewIndex(3)
	for _, s := range []string{"a", "b", "c", "d"} {
		x.Note([]byte(s))
	}
	if x.Len() != 3 || x.Count([]byte("a")) != 0 {
		t.Fatalf("oldest token should be gone: len=%d", x.Len())
	}
	got := x.Recent(10)
	if len(got) != 3 || string(got[0]) != "d" || string(got[2]) != "b" {
		t.Fatalf("unexpected order: %q", got)
	}
}

func TestIndexCountsRepeats(t *testing.T) {
	x := NewIndex(2)
	x.Note([]byte("a"))
	x.Note([]byte("b"))
	x.Note([]byte("a"))
	if x.Count([]byte("a")) != 2 || string(x.Recent(1)[0]) != "a" {
		t.Fatal("a repeat should count and move to the front")
	}
}

func TestRouterRules(t *testing.T) {
	r := NewRouter(map[string]string{"/api": "core"}, []string{"/api"}, 4)
	cases := []struct {
		head    string
		backend string
		allowed bool
	}{
		{"GET /api/x HTTP/1.1\r\n\r\n", "core", true},
		{"TRACE /api/x HTTP/1.1\r\n\r\n", "core", false},
		{"GET /other HTTP/1.1\r\n\r\n", "default", true},
		{"GET /api/../x HTTP/1.1\r\n\r\n", "", false},
	}
	for _, c := range cases {
		got, err := r.Handle(frameOf(c.head))
		if err != nil || got.Backend != c.backend || got.Allowed != c.allowed {
			t.Errorf("%q: got %+v, %v", c.head, got, err)
		}
	}
}

func TestShardIsStable(t *testing.T) {
	if Shard([]byte("t-1"), 8) != Shard([]byte("t-1"), 8) || Shard([]byte("t-1"), 8) < 0 {
		t.Fatal("shard must be stable and non-negative")
	}
}
