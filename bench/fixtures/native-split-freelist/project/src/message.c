#include <string.h>

#include "types.h"

static uint32_t mix(uint32_t h, uint8_t b)
{
    h ^= b;
    h *= 16777619u;
    return h;
}

static uint32_t checksum(const message *m)
{
    uint32_t h = 2166136261u;
    for (uint32_t i = 0; i < m->len; i++)
        h = mix(h, m->payload[i]);
    return h;
}

message *message_new(uint32_t id, uint16_t kind, uint32_t len, uint64_t seed)
{
    message *m = pool_alloc(sizeof *m + len);
    if (m == NULL)
        return NULL;
    m->id = id;
    m->kind = kind;
    m->flags = 0;
    m->len = len;
    uint64_t x = seed | 1u;
    for (uint32_t i = 0; i < len; i++) {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        m->payload[i] = (uint8_t)x;
    }
    m->sum = checksum(m);
    return m;
}

void message_free(message *m)
{
    pool_free(m);
}

int message_verify(const message *m)
{
    return m->sum == checksum(m) ? 0 : -1;
}
