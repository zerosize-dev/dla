#include <stdio.h>

#include "types.h"

void report_summary(const gateway *g, unsigned long iterations)
{
    unsigned long bad = 0;
    unsigned long long bytes = 0;
    for (size_t i = 0; i < g->capacity; i++) {
        const message *m = g->inflight[i];
        if (m == NULL)
            continue;
        bytes += m->len;
        if (message_verify(m) != 0)
            bad++;
    }
    printf("ticks=%lu accepted=%lu acknowledged=%lu bulk=%lu in_flight=%zu payload_bytes=%llu corrupt=%lu\n",
           iterations, g->accepted, g->acknowledged, g->bulk, g->count, bytes, bad);
}
