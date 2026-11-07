#include <stdlib.h>

#include "ring.h"

struct ring {
    const void **items;
    size_t slots;
    size_t next;
    size_t count;
};

ring *ring_create(size_t slots)
{
    ring *r = calloc(1, sizeof *r);
    if (r == NULL)
        return NULL;
    r->items = calloc(slots, sizeof *r->items);
    if (r->items == NULL) {
        free(r);
        return NULL;
    }
    r->slots = slots;
    return r;
}

void ring_destroy(ring *r)
{
    free(r->items);
    free(r);
}

void ring_put(ring *r, const void *item)
{
    r->items[r->next] = item;
    r->next = (r->next + 1) % r->slots;
    if (r->count < r->slots)
        r->count++;
}

size_t ring_count(const ring *r)
{
    return r->count;
}

const void *ring_get(const ring *r, size_t i)
{
    if (i >= r->count)
        return NULL;
    size_t first = (r->next + r->slots - r->count) % r->slots;
    return r->items[(first + i) % r->slots];
}
