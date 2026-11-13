package trace

import (
	"context"
	"testing"
)

func TestStartLinksToParent(t *testing.T) {
	root := Start(context.Background(), "request")
	child := Start(root, "fetch", Attr{"pair", "A/B"})
	if From(child).Parent != From(root) || From(root).Parent != nil {
		t.Fatal("child span should point at the root span")
	}
	if From(context.Background()) != nil {
		t.Fatal("an empty context carries no span")
	}
}

func TestPathLimitsDepth(t *testing.T) {
	ctx := context.Background()
	for _, n := range []string{"a", "b", "c", "d"} {
		ctx = Start(ctx, n)
	}
	if got := Path(ctx, 3); got != "b/c/d" {
		t.Fatalf("unexpected path %q", got)
	}
	if got := Path(context.Background(), 3); got != "" {
		t.Fatalf("unexpected path %q", got)
	}
}

func TestSpansGetDistinctIDs(t *testing.T) {
	a := From(Start(context.Background(), "x"))
	b := From(Start(context.Background(), "x"))
	if a.ID == b.ID {
		t.Fatal("ids must differ")
	}
}
