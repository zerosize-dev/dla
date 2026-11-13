package edge

import "bytes"

// Request is the parsed head of a frame. Its fields point into the frame.
type Request struct {
	Method  []byte
	Target  []byte
	Tenant  []byte
	Request []byte
}

// ParseRequest reads the request line and the headers the router needs.
func ParseRequest(f *Frame) (Request, error) {
	data := f.Bytes()
	end := bytes.IndexByte(data, '\n')
	if end < 0 {
		return Request{}, ErrShortFrame
	}
	line := bytes.TrimRight(data[:end], "\r")
	parts := bytes.Fields(line)
	if len(parts) < 2 {
		return Request{}, ErrShortFrame
	}
	req := Request{Method: parts[0], Target: parts[1]}
	rest := data[end+1:]
	for len(rest) > 0 {
		nl := bytes.IndexByte(rest, '\n')
		if nl < 0 {
			break
		}
		h := bytes.TrimRight(rest[:nl], "\r")
		rest = rest[nl+1:]
		if len(h) == 0 {
			break
		}
		name, value, ok := bytes.Cut(h, []byte(": "))
		if !ok {
			continue
		}
		switch string(name) {
		case "X-Tenant":
			req.Tenant = value
		case "X-Request-Id":
			req.Request = value
		}
	}
	return req, nil
}
