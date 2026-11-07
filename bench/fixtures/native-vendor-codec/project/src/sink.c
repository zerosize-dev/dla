#include "sink.h"

int sink_init(sink *s, size_t keep, size_t frame_cap)
{
    s->recent = ring_create(keep, frame_cap);
    s->frames = 0;
    s->bytes = 0;
    s->checksum = 2166136261u;
    return s->recent == NULL ? -1 : 0;
}

void sink_close(sink *s)
{
    ring_destroy(s->recent);
    s->recent = NULL;
}

void sink_push(sink *s, const uint8_t *data, size_t len)
{
    ring_put(s->recent, data, len);
    s->frames++;
    s->bytes += len;
    for (size_t i = 0; i < len; i += 61)
        s->checksum = (s->checksum ^ data[i]) * 16777619u;
}
