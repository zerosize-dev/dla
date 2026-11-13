package edge

import (
	"hash/crc32"
	"strings"
)

// Route says where a request goes.
type Route struct {
	Backend string
	Allowed bool
}

// Router maps tenants and path prefixes to backends and keeps the index
// behind the diagnostics endpoint up to date.
type Router struct {
	rules []rule
	seen  *Index
}

type rule struct {
	prefix  string
	backend string
}

// NewRouter returns a router with the given prefix rules, in match order,
// and an index of the last indexSize request ids.
func NewRouter(rules map[string]string, order []string, indexSize int) *Router {
	r := &Router{seen: NewIndex(indexSize)}
	for _, p := range order {
		r.rules = append(r.rules, rule{prefix: p, backend: rules[p]})
	}
	return r
}

// Index returns the diagnostics index.
func (r *Router) Index() *Index {
	return r.seen
}

// Handle routes one frame.
func (r *Router) Handle(f *Frame) (Route, error) {
	req, err := ParseRequest(f)
	if err != nil {
		return Route{}, err
	}
	if len(req.Request) > 0 {
		r.seen.Note(req.Request)
	}
	target := string(req.Target)
	if strings.Contains(target, "..") {
		return Route{Allowed: false}, nil
	}
	for _, ru := range r.rules {
		if strings.HasPrefix(target, ru.prefix) {
			return Route{Backend: ru.backend, Allowed: string(req.Method) != "TRACE"}, nil
		}
	}
	return Route{Backend: "default", Allowed: true}, nil
}

// Shard picks a stable shard for a tenant.
func Shard(tenant []byte, shards int) int {
	return int(crc32.ChecksumIEEE(tenant)) % shards
}
