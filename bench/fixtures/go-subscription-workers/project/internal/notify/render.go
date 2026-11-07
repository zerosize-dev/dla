package notify

import "strings"

const maxLine = 160

// Renderer turns messages into the text that is handed to a sink. Each topic
// has a layout with {title} and {body} placeholders.
type Renderer struct {
	layouts  map[string]string
	fallback string
}

// NewRenderer builds a renderer with the given topic layouts.
func NewRenderer(layouts map[string]string) *Renderer {
	cp := make(map[string]string, len(layouts))
	for k, v := range layouts {
		cp[k] = v
	}
	return &Renderer{layouts: cp, fallback: "{title}: {body}"}
}

// Knows reports whether the topic has a layout.
func (r *Renderer) Knows(topic string) bool {
	_, ok := r.layouts[topic]
	return ok
}

// Render appends the formatted message to dst and returns the result.
func (r *Renderer) Render(dst []byte, m Message) []byte {
	layout, ok := r.layouts[m.Topic]
	if !ok {
		layout = r.fallback
	}
	title := clip(m.Title, 48)
	body := clip(m.Body, maxLine)
	rest := layout
	for len(rest) > 0 {
		i := strings.IndexByte(rest, '{')
		if i < 0 {
			dst = append(dst, rest...)
			break
		}
		dst = append(dst, rest[:i]...)
		rest = rest[i:]
		switch {
		case strings.HasPrefix(rest, "{title}"):
			dst = append(dst, title...)
			rest = rest[len("{title}"):]
		case strings.HasPrefix(rest, "{body}"):
			dst = append(dst, body...)
			rest = rest[len("{body}"):]
		default:
			dst = append(dst, '{')
			rest = rest[1:]
		}
	}
	if m.Priority == Urgent {
		dst = append(dst, " [urgent]"...)
	}
	return dst
}

func clip(s string, n int) string {
	if len(s) <= n {
		return s
	}
	return s[:n-3] + "..."
}
