#include <string.h>

#include "types.h"

static const char *const hubs[HUB_COUNT] = {
    "AMS3", "BRU1", "CDG2", "DUB1", "FRA4", "LIS1", "MAD2", "VIE1"
};

const char *hub_name(unsigned index)
{
    return index < HUB_COUNT ? hubs[index] : NULL;
}

uint32_t manifest_sum(const uint8_t *data, size_t len)
{
    uint32_t a = 1, b = 0;
    for (size_t i = 0; i < len; i++) {
        a = (a + data[i]) % 65521u;
        b = (b + a) % 65521u;
    }
    return (b << 16) | a;
}

static uint64_t next_rand(feed *f)
{
    f->state ^= f->state << 13;
    f->state ^= f->state >> 7;
    f->state ^= f->state << 17;
    return f->state;
}

void feed_init(feed *f, uint64_t seed, uint32_t damage_every, uint32_t unknown_every)
{
    f->state = seed ? seed : 88172645463325252ULL;
    f->next_seq = 1;
    f->damage_every = damage_every;
    f->unknown_every = unknown_every;
}

void feed_next(feed *f, parcel *out)
{
    uint64_t r = next_rand(f);
    memset(out, 0, sizeof *out);
    out->seq = f->next_seq++;
    out->lane = (uint16_t)(r % 12u);
    out->manifest_len = (uint16_t)(32u + (r >> 8) % (MANIFEST_MAX - 32u));
    for (uint16_t i = 0; i < out->manifest_len; i++)
        out->manifest[i] = (uint8_t)(next_rand(f) >> 16);
    out->sum = manifest_sum(out->manifest, out->manifest_len);

    const char *hub = hubs[(r >> 24) % HUB_COUNT];
    if (f->unknown_every != 0 && out->seq % f->unknown_every == 0)
        hub = "ZZZ9";
    strncpy(out->hub, hub, sizeof out->hub - 1);

    if (f->damage_every != 0 && out->seq % f->damage_every == 0)
        out->manifest[0] ^= 0x55;
}
