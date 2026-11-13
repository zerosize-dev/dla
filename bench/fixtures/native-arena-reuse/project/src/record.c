#include <stdlib.h>

#include "types.h"

record *record_new(uint32_t id, int64_t cents, uint32_t memo_len, uint64_t seed)
{
    record *r = malloc(sizeof *r + memo_len);
    if (r == NULL)
        return NULL;
    r->id = id;
    r->cents = cents;
    r->memo_len = memo_len;
    uint64_t x = seed | 1u;
    for (uint32_t i = 0; i < memo_len; i++) {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        r->memo[i] = (char)('a' + (x % 26u));
    }
    return r;
}

void record_free(record *r)
{
    free(r);
}

uint32_t record_digest(const record *r)
{
    uint32_t h = 2166136261u ^ r->id;
    for (uint32_t i = 0; i < r->memo_len; i++) {
        h ^= (uint8_t)r->memo[i];
        h *= 16777619u;
    }
    return h;
}
