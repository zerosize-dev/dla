package render

import (
	"hash/fnv"
	"sort"
)

// Result is the outcome of rendering one batch.
type Result struct {
	Invoices int
	Bytes    int
	Digest   uint64
}

type rendered struct {
	buf   []byte
	total int64
}

// Renderer renders batches of invoices. Documents of a batch are indexed by
// invoice ID while the batch is assembled, then the index is emptied.
type Renderer struct {
	pool  *BufferPool
	index map[string]*rendered
}

// NewRenderer returns a renderer that takes its buffers from pool.
func NewRenderer(pool *BufferPool) *Renderer {
	return &Renderer{pool: pool, index: make(map[string]*rendered)}
}

// RenderBatch renders every invoice and folds the documents into a digest in
// invoice ID order. Invoices with a repeated ID replace the earlier one.
func (r *Renderer) RenderBatch(batch []Invoice) Result {
	for _, inv := range batch {
		if old, ok := r.index[inv.ID]; ok {
			r.pool.Put(old.buf)
		}
		buf := appendInvoice(r.pool.Get(), inv)
		r.index[inv.ID] = &rendered{buf: buf, total: inv.Total()}
	}

	ids := make([]string, 0, len(r.index))
	for id := range r.index {
		ids = append(ids, id)
	}
	sort.Strings(ids)

	h := fnv.New64a()
	var res Result
	for _, id := range ids {
		doc := r.index[id]
		h.Write(doc.buf)
		res.Bytes += len(doc.buf)
		res.Invoices++
	}
	res.Digest = h.Sum64()

	for id, doc := range r.index {
		r.pool.Put(doc.buf)
		delete(r.index, id)
	}
	return res
}
