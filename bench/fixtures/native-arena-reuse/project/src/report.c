#include <malloc.h>
#include <stdio.h>

#include "types.h"

void report_summary(const settlement *s, unsigned long iterations)
{
    printf("batches=%lu records=%lu total_cents=%lld digest=%08x carried=%zu\n",
           iterations, s->records, (long long)s->total_cents, (unsigned)s->digest,
           carry_held(&s->carried));
}

void report_heap(void)
{
    struct mallinfo2 mi = mallinfo2();
    fprintf(stderr, "heap: arena=%zu in_use=%zu free=%zu\n", mi.arena, mi.uordblks,
            mi.fordblks);
}
