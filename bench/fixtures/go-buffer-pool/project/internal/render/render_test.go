package render

import (
	"strings"
	"testing"
)

func sample(id string) Invoice {
	return Invoice{
		ID: id, Customer: "acme", Currency: "EUR",
		Lines: []Line{
			{SKU: "A", Desc: "widget", Quantity: 2, UnitCents: 1050},
			{SKU: "B", Desc: "gadget", Quantity: 1, UnitCents: 5},
		},
	}
}

func TestTotalAndMoney(t *testing.T) {
	if got := sample("x").Total(); got != 2105 {
		t.Fatalf("total = %d", got)
	}
	if got := string(appendMoney(nil, 2105)); got != "21.05" {
		t.Fatalf("money = %q", got)
	}
	if got := string(appendMoney(nil, -7)); got != "-0.07" {
		t.Fatalf("money = %q", got)
	}
}

func TestAppendInvoice(t *testing.T) {
	out := string(appendInvoice(nil, sample("INV-1")))
	for _, want := range []string{"INVOICE INV-1", "1. A widget x2 @ 10.50", "TOTAL 21.05 EUR"} {
		if !strings.Contains(out, want) {
			t.Fatalf("missing %q in %q", want, out)
		}
	}
}

func TestPoolReusesBuffers(t *testing.T) {
	p := NewBufferPool()
	a := p.Get()
	p.Put(a)
	b := p.Get()
	if created, idle := p.Stats(); created != 1 || idle != 0 || len(b) != 0 {
		t.Fatalf("created=%d idle=%d len=%d", created, idle, len(b))
	}
}

func TestRenderBatchDigestIsOrderIndependent(t *testing.T) {
	a := NewRenderer(NewBufferPool()).RenderBatch([]Invoice{sample("a"), sample("b")})
	b := NewRenderer(NewBufferPool()).RenderBatch([]Invoice{sample("b"), sample("a")})
	if a != b || a.Invoices != 2 {
		t.Fatalf("results differ: %+v %+v", a, b)
	}
}

func TestRepeatedIDReplaces(t *testing.T) {
	r := NewRenderer(NewBufferPool())
	res := r.RenderBatch([]Invoice{sample("a"), sample("a")})
	if res.Invoices != 1 {
		t.Fatalf("invoices = %d", res.Invoices)
	}
}
