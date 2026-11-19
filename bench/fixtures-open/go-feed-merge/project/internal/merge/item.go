// Package merge combines the documents of several feeds into one timeline.
package merge

import (
	"bytes"
	"strconv"
)

// Item is one entry of a feed document.
type Item struct {
	ID       string
	Time     int64
	Headline []byte
	Feed     string
}

// Parse reads the lines of a feed document. A line is "id|time|headline";
// lines that do not parse are skipped. The headlines point into doc.
func Parse(feed string, doc []byte) []Item {
	var items []Item
	for len(doc) > 0 {
		line := doc
		if i := bytes.IndexByte(doc, '\n'); i >= 0 {
			line, doc = doc[:i], doc[i+1:]
		} else {
			doc = nil
		}
		fields := bytes.SplitN(line, []byte("|"), 3)
		if len(fields) != 3 {
			continue
		}
		ts, err := strconv.ParseInt(string(fields[1]), 10, 64)
		if err != nil {
			continue
		}
		items = append(items, Item{ID: string(fields[0]), Time: ts, Headline: fields[2], Feed: feed})
	}
	return items
}
