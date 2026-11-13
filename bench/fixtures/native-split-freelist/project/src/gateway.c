#include <stdlib.h>

#include "types.h"

/* Status frames are 40 to 200 bytes; every tenth message is a bulk record of
 * 1.5 to 3 KiB. */
#define STATUS_MIN 40u
#define STATUS_SPAN 160u
#define BULK_MIN 1536u
#define BULK_SPAN 1536u

uint64_t gateway_rand(gateway *g)
{
    g->rng ^= g->rng << 13;
    g->rng ^= g->rng >> 7;
    g->rng ^= g->rng << 17;
    return g->rng;
}

int gateway_init(gateway *g, size_t capacity, uint64_t seed)
{
    g->inflight = calloc(capacity, sizeof *g->inflight);
    if (g->inflight == NULL)
        return -1;
    g->capacity = capacity;
    g->count = 0;
    g->rng = seed ? seed : 88172645463325252ULL;
    g->next_id = 1;
    g->accepted = 0;
    g->acknowledged = 0;
    g->bulk = 0;
    return 0;
}

void gateway_destroy(gateway *g)
{
    for (size_t i = 0; i < g->capacity; i++) {
        if (g->inflight[i] != NULL)
            message_free(g->inflight[i]);
    }
    free(g->inflight);
    g->inflight = NULL;
    g->count = 0;
}

/* One tick: the archive acknowledges one of the buffered messages, and a
 * device sends a new one that takes its place. */
void gateway_step(gateway *g)
{
    size_t slot = (size_t)(gateway_rand(g) % g->capacity);
    if (g->inflight[slot] != NULL) {
        message_free(g->inflight[slot]);
        g->inflight[slot] = NULL;
        g->count--;
        g->acknowledged++;
    }

    uint64_t r = gateway_rand(g);
    uint16_t kind = (r % 10u == 0) ? KIND_BULK : KIND_STATUS;
    uint32_t len = kind == KIND_BULK
                       ? BULK_MIN + (uint32_t)((r >> 8) % BULK_SPAN)
                       : STATUS_MIN + (uint32_t)((r >> 8) % STATUS_SPAN);
    message *m = message_new(g->next_id++, kind, len, r);
    if (m == NULL)
        return;
    g->inflight[slot] = m;
    g->count++;
    g->accepted++;
    if (kind == KIND_BULK)
        g->bulk++;
}
