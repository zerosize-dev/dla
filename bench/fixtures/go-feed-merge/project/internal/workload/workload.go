// Package workload generates deterministic upstream feeds for the merger.
package workload

import (
	"bytes"
	"fmt"

	"example.com/feedmerged/internal/fetcher"
	"example.com/feedmerged/internal/indexer"
	"example.com/feedmerged/internal/merge"
)

// Summary is what a replay produced.
type Summary struct {
	Rounds   int
	Items    int
	Unique   int
	Checksum uint32
}

// FeedNames are the upstream feeds the replay reads.
var FeedNames = []string{"wire", "markets", "weather"}

type rng uint64

func (r *rng) next() uint64 {
	*r = *r*6364136223846793005 + 1442695040888963407
	return uint64(*r >> 33)
}

// DocSize is the size of every upstream document.
const DocSize = 6144

// Upstreams returns the synthetic upstream feeds.
func Upstreams() map[string]fetcher.Upstream {
	ups := make(map[string]fetcher.Upstream)
	for k, name := range FeedNames {
		k, name := k, name
		ups[name] = func(seq int) []byte {
			r := rng(uint64(seq)*7919 + uint64(k)*104729 + 3)
			b := bytes.NewBuffer(make([]byte, 0, DocSize))
			for line := 0; b.Len() < DocSize-64; line++ {
				fmt.Fprintf(b, "%s-%d-%d|%d|%s item %x\n", name, seq, line, 1700000000+int64(seq)*60+int64(r.next()%50), name, r.next())
			}
			for b.Len() < DocSize {
				b.WriteByte(' ')
			}
			return b.Bytes()
		}
	}
	return ups
}

// NewMerger builds the merger used by Run.
func NewMerger() *merge.Merger {
	return merge.NewMerger(fetcher.New(Upstreams()), FeedNames, indexer.New(120000), 3)
}

// Run replays n merge rounds.
func Run(n int) (Summary, error) {
	m := NewMerger()
	var sum Summary
	for i := 0; i < n; i++ {
		res, err := m.Round()
		if err != nil {
			return sum, fmt.Errorf("round %d: %w", i, err)
		}
		sum.Rounds++
		sum.Items += res.Items
		sum.Unique += res.Unique
		for _, it := range res.Top {
			sum.Checksum = sum.Checksum*31 + uint32(it.Time) + uint32(len(it.Headline))
		}
	}
	return sum, nil
}
