package render

import "sync"

const initialBuffer = 1024

// BufferPool hands out byte buffers for rendering and takes them back when
// the caller is done, so the same backing arrays serve many batches.
type BufferPool struct {
	mu      sync.Mutex
	free    [][]byte
	created int
}

// NewBufferPool returns an empty pool.
func NewBufferPool() *BufferPool {
	return &BufferPool{}
}

// Get returns an empty buffer.
func (p *BufferPool) Get() []byte {
	p.mu.Lock()
	defer p.mu.Unlock()
	if n := len(p.free); n > 0 {
		buf := p.free[n-1]
		p.free[n-1] = nil
		p.free = p.free[:n-1]
		return buf[:0]
	}
	p.created++
	return make([]byte, 0, initialBuffer)
}

// Put returns a buffer to the pool.
func (p *BufferPool) Put(buf []byte) {
	p.mu.Lock()
	p.free = append(p.free, buf)
	p.mu.Unlock()
}

// Stats reports how many buffers the pool has created and how many are
// currently idle.
func (p *BufferPool) Stats() (created, idle int) {
	p.mu.Lock()
	defer p.mu.Unlock()
	return p.created, len(p.free)
}
