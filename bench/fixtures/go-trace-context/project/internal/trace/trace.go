// Package trace carries a small span record through a context.Context so log
// lines and upstream calls can be tied back to the work that started them.
package trace

import (
	"context"
	"fmt"
	"sync/atomic"
)

// Attr is one key/value pair on a span.
type Attr struct {
	Key   string
	Value string
}

// Span describes a unit of work.
type Span struct {
	ID     uint64
	Name   string
	Parent *Span
	Depth  int
	Attrs  []Attr
}

type spanKey struct{}

var nextID atomic.Uint64

// Start returns a context that carries a new span, a child of the span in
// ctx when there is one.
func Start(ctx context.Context, name string, attrs ...Attr) context.Context {
	s := &Span{ID: nextID.Add(1), Name: name, Parent: From(ctx)}
	if s.Parent != nil {
		s.Depth = s.Parent.Depth + 1
	}
	s.Attrs = append(s.Attrs, attrs...)
	s.Attrs = append(s.Attrs,
		Attr{"span.name", name},
		Attr{"span.seq", fmt.Sprint(s.ID)},
		Attr{"span.depth", fmt.Sprint(s.Depth)},
		Attr{"span.label", fmt.Sprintf("%s#%d", name, s.ID)})
	return context.WithValue(ctx, spanKey{}, s)
}

// From returns the span carried by ctx, or nil.
func From(ctx context.Context) *Span {
	s, _ := ctx.Value(spanKey{}).(*Span)
	return s
}

// Path returns the names of the spans from the root of the span in ctx down
// to it, joined with "/", limited to the last depth names.
func Path(ctx context.Context, depth int) string {
	var names []string
	for s := From(ctx); s != nil && len(names) < depth; s = s.Parent {
		names = append([]string{s.Name}, names...)
	}
	out := ""
	for i, n := range names {
		if i > 0 {
			out += "/"
		}
		out += n
	}
	return out
}
