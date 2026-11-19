package edge

import "container/list"

// Index remembers the most recent request tokens so the diagnostics endpoint
// can show what the proxy has been seeing without touching the hot path.
type Index struct {
	limit   int
	order   *list.List
	byToken map[string]*list.Element
}

type seen struct {
	token []byte
	count int
}

// NewIndex returns an index that keeps at most limit distinct tokens.
func NewIndex(limit int) *Index {
	return &Index{limit: limit, order: list.New(), byToken: make(map[string]*list.Element, limit)}
}

// Note records one sighting of token. The oldest token is dropped when the
// index is full.
func (x *Index) Note(token []byte) {
	if el, ok := x.byToken[string(token)]; ok {
		el.Value.(*seen).count++
		x.order.MoveToFront(el)
		return
	}
	if x.order.Len() >= x.limit {
		old := x.order.Back()
		x.order.Remove(old)
		delete(x.byToken, string(old.Value.(*seen).token))
	}
	x.byToken[string(token)] = x.order.PushFront(&seen{token: token, count: 1})
}

// Len reports how many tokens are remembered.
func (x *Index) Len() int {
	return x.order.Len()
}

// Recent returns up to n tokens, newest first.
func (x *Index) Recent(n int) [][]byte {
	var out [][]byte
	for el := x.order.Front(); el != nil && len(out) < n; el = el.Next() {
		out = append(out, el.Value.(*seen).token)
	}
	return out
}

// Count returns how often token has been seen while it was remembered.
func (x *Index) Count(token []byte) int {
	if el, ok := x.byToken[string(token)]; ok {
		return el.Value.(*seen).count
	}
	return 0
}
