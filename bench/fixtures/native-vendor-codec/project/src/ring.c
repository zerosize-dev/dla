#include <stdlib.h>
#include <string.h>

#include "ring.h"

struct ring {
    uint8_t *data;
    size_t *lens;
    size_t slots;
    size_t slot_size;
    size_t next;
    size_t count;
};

ring *ring_create(size_t slots, size_t slot_size)
{
    ring *r = calloc(1, sizeof *r);
    if (r == NULL)
        return NULL;
    r->data = calloc(slots, slot_size);
    r->lens = calloc(slots, sizeof *r->lens);
    if (r->data == NULL || r->lens == NULL) {
        free(r->data);
        free(r->lens);
        free(r);
        return NULL;
    }
    r->slots = slots;
    r->slot_size = slot_size;
    return r;
}

void ring_destroy(ring *r)
{
    free(r->data);
    free(r->lens);
    free(r);
}

void ring_put(ring *r, const uint8_t *data, size_t len)
{
    if (len > r->slot_size)
        len = r->slot_size;
    memcpy(r->data + r->next * r->slot_size, data, len);
    r->lens[r->next] = len;
    r->next = (r->next + 1) % r->slots;
    if (r->count < r->slots)
        r->count++;
}

size_t ring_count(const ring *r)
{
    return r->count;
}

size_t ring_get(const ring *r, size_t i, uint8_t *out, size_t cap)
{
    if (i >= r->count)
        return 0;
    size_t first = (r->next + r->slots - r->count) % r->slots;
    size_t slot = (first + i) % r->slots;
    size_t len = r->lens[slot];
    if (len > cap)
        len = cap;
    memcpy(out, r->data + slot * r->slot_size, len);
    return len;
}
