#include <stdio.h>

#include "types.h"

void report_summary(const broker *b, unsigned long iterations)
{
    uint64_t digest = 0;
    for (size_t i = 0; i < b->active_len; i++)
        digest = digest * 1099511628211ULL + b->active[i];
    printf("ticks=%lu granted=%lu released=%lu active=%zu digest=%016llx\n",
           iterations, b->granted, b->released, lease_table_live(&b->table),
           (unsigned long long)digest);
}
