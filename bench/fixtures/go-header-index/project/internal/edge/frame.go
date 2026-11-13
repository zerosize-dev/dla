package edge

import (
	"errors"
	"io"
)

// FrameSize is the size of the buffer one connection read fills.
const FrameSize = 16 << 10

// ErrShortFrame is returned when a frame has no complete request line.
var ErrShortFrame = errors.New("edge: frame has no request line")

// Frame is one read from a client connection: a request line, header lines
// and the start of the body, in a buffer of FrameSize bytes.
type Frame struct {
	buf []byte
	n   int
}

// ReadFrame reads the next frame from r into a fresh buffer.
func ReadFrame(r io.Reader) (*Frame, error) {
	buf := make([]byte, FrameSize)
	n, err := io.ReadFull(r, buf)
	if err != nil && !errors.Is(err, io.ErrUnexpectedEOF) {
		return nil, err
	}
	if n == 0 {
		return nil, io.EOF
	}
	return &Frame{buf: buf, n: n}, nil
}

// Bytes returns the filled part of the frame. The caller must not modify it.
func (f *Frame) Bytes() []byte {
	return f.buf[:f.n]
}
