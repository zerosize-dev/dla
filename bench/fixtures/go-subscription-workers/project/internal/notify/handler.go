package notify

import "sort"

// Handler serves notification requests.
type Handler struct {
	hub      *Hub
	maxItems int
}

// NewHandler returns a handler that accepts up to maxItems items per request.
func NewHandler(h *Hub, maxItems int) *Handler {
	return &Handler{hub: h, maxItems: maxItems}
}

// Serve delivers every accepted item of the request to its recipient.
func (h *Handler) Serve(req Request) (Response, error) {
	sub, err := h.hub.Open(req.Recipient, req.Topic)
	if err != nil {
		return Response{}, err
	}
	items := append([]Item(nil), req.Items...)
	sort.SliceStable(items, func(i, j int) bool { return items[i].Priority > items[j].Priority })

	var resp Response
	for _, it := range items {
		if resp.Delivered >= h.maxItems || it.Title == "" {
			resp.Skipped++
			continue
		}
		if err := sub.Push(Message{Title: it.Title, Body: it.Body, Priority: it.Priority}); err != nil {
			return resp, err
		}
		resp.Delivered++
	}
	if err := sub.Flush(); err != nil {
		return resp, err
	}
	return resp, nil
}
