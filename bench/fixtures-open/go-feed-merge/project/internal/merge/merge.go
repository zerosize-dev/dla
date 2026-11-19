package merge

import (
	"sort"

	"example.com/feedmerged/internal/fetcher"
	"example.com/feedmerged/internal/indexer"
)

// Merger reads every feed once per round and builds the timeline.
type Merger struct {
	fetch *fetcher.Fetcher
	feeds []string
	index *indexer.Indexer
	seen  map[string]bool
	limit int
}

// NewMerger returns a merger over feeds that keeps timelines of at most
// limit items and remembers headlines in index.
func NewMerger(f *fetcher.Fetcher, feeds []string, index *indexer.Indexer, limit int) *Merger {
	return &Merger{fetch: f, feeds: feeds, index: index, seen: make(map[string]bool), limit: limit}
}

// Result is the outcome of one round.
type Result struct {
	Items  int
	Unique int
	Top    []Item
}

// Round reads every feed, drops items already seen in this window and
// returns the newest ones.
func (m *Merger) Round() (Result, error) {
	var all []Item
	for _, name := range m.feeds {
		sess, err := m.fetch.Open(name)
		if err != nil {
			return Result{}, err
		}
		all = append(all, Parse(sess.Feed(), sess.Read())...)
	}
	res := Result{Items: len(all)}
	var fresh []Item
	for _, it := range all {
		if m.seen[it.ID] {
			continue
		}
		m.seen[it.ID] = true
		fresh = append(fresh, it)
	}
	if len(m.seen) > 4096 {
		m.seen = make(map[string]bool)
	}
	sort.Slice(fresh, func(i, j int) bool { return fresh[i].Time > fresh[j].Time })
	res.Unique = len(fresh)
	if len(fresh) > m.limit {
		fresh = fresh[:m.limit]
	}
	for _, it := range fresh {
		m.index.Remember(it.Headline)
	}
	res.Top = fresh
	return res, nil
}
