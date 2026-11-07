package notify

import (
	"strings"
	"testing"
)

type captureSink struct{ got []string }

func (c *captureSink) Deliver(recipient string, text []byte) {
	c.got = append(c.got, recipient+"|"+string(text))
}

func newTestHub() (*Hub, *captureSink) {
	sink := &captureSink{}
	r := NewRenderer(map[string]string{"billing": "[b] {title} - {body}"})
	return NewHub(r, sink), sink
}

func TestRenderLayout(t *testing.T) {
	r := NewRenderer(map[string]string{"x": "<{title}|{body}>"})
	got := string(r.Render(nil, Message{Topic: "x", Title: "a", Body: "b", Priority: Urgent}))
	if got != "<a|b> [urgent]" {
		t.Fatalf("unexpected render: %q", got)
	}
}

func TestRenderClips(t *testing.T) {
	r := NewRenderer(nil)
	got := string(r.Render(nil, Message{Title: "t", Body: strings.Repeat("z", 400)}))
	if len(got) > 200 {
		t.Fatalf("body was not clipped: %d bytes", len(got))
	}
}

func TestOpenValidation(t *testing.T) {
	h, _ := newTestHub()
	if _, err := h.Open("  ", "billing"); err != ErrNoRecipient {
		t.Fatalf("want ErrNoRecipient, got %v", err)
	}
	if _, err := h.Open("a", "nope"); err != ErrNoTopic {
		t.Fatalf("want ErrNoTopic, got %v", err)
	}
}

func TestSubscriptionDeliversInOrder(t *testing.T) {
	h, sink := newTestHub()
	sub, err := h.Open("Ann", "billing")
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Close()
	for _, title := range []string{"one", "two", "three"} {
		if err := sub.Push(Message{Title: title, Body: "x"}); err != nil {
			t.Fatal(err)
		}
	}
	if err := sub.Flush(); err != nil {
		t.Fatal(err)
	}
	if len(sink.got) != 3 || sink.got[0] != "ann|[b] one - x" || sink.got[2] != "ann|[b] three - x" {
		t.Fatalf("unexpected deliveries: %v", sink.got)
	}
}

func TestPushAfterClose(t *testing.T) {
	h, _ := newTestHub()
	sub, _ := h.Open("ann", "billing")
	sub.Close()
	sub.Close()
	if err := sub.Push(Message{Title: "late"}); err != ErrClosed {
		t.Fatalf("want ErrClosed, got %v", err)
	}
}

func TestHandlerLimitsAndOrders(t *testing.T) {
	h, sink := newTestHub()
	handler := NewHandler(h, 2)
	resp, err := handler.Serve(Request{
		Recipient: "bob",
		Topic:     "billing",
		Items: []Item{
			{Title: "low", Priority: Low},
			{Title: "urgent", Priority: Urgent},
			{Title: "", Priority: Urgent},
			{Title: "normal", Priority: Normal},
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	if resp.Delivered != 2 || resp.Skipped != 2 {
		t.Fatalf("unexpected response: %+v", resp)
	}
	if !strings.Contains(sink.got[0], "urgent") {
		t.Fatalf("urgent item should go first: %v", sink.got)
	}
}

func TestMemorySinkTotals(t *testing.T) {
	var s MemorySink
	s.Deliver("a", []byte("xy"))
	s.Deliver("b", []byte("z"))
	n, b, sum := s.Totals()
	if n != 2 || b != 3 || sum == 0 {
		t.Fatalf("unexpected totals: %d %d %d", n, b, sum)
	}
}
