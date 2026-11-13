#include <stdio.h>

#include "types.h"

void report_summary(const router *r, unsigned long iterations)
{
    printf("parcels=%lu routed=%lu rejected=%lu hubs=", iterations, r->routed, r->rejected);
    for (unsigned i = 0; i < HUB_COUNT; i++)
        printf("%s%lu", i ? "," : "", r->per_hub[i]);
    printf("\n");
}
