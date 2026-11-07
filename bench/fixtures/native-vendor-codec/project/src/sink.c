#include "sink.h"

int sink_init(sink *s, size_t keep)
{
    s->replay = ring_create(keep);
    s->frames = 0;
    s->bytes = 0;
    s->checksum = 2166136261u;
    return s->replay == NULL ? -1 : 0;
}

void sink_close(sink *s)
{
    ring_destroy(s->replay);
    s->replay = NULL;
}

void sink_push(sink *s, const acme_frame *f)
{
    s->frames++;
    s->bytes += f->size;
    for (size_t i = 0; i < f->size; i += 61)
        s->checksum = (s->checksum ^ f->data[i]) * 16777619u;
    ring_put(s->replay, f);
}

const acme_frame *sink_replay(const sink *s, size_t i)
{
    return ring_get(s->replay, i);
}
