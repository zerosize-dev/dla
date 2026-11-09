#include <string.h>

#include "frame.h"

static void store_u32(uint8_t *p, uint32_t v)
{
    p[0] = (uint8_t)v;
    p[1] = (uint8_t)(v >> 8);
    p[2] = (uint8_t)(v >> 16);
    p[3] = (uint8_t)(v >> 24);
}

void synth_init(synth_source *s, uint64_t seed, uint32_t damage_every)
{
    s->state = seed ? seed : 0x9E3779B97F4A7C15ull;
    s->next_seq = 0;
    s->damage_every = damage_every;
}

uint32_t synth_rand(synth_source *s)
{
    uint64_t x = s->state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    s->state = x;
    return (uint32_t)(x >> 16);
}

/* Builds one frame of 4..32 KiB of banded "scanline" data. */
size_t synth_next(synth_source *s, uint8_t *wire, size_t cap)
{
    static uint8_t raw[FRAME_MAX_DECODED];
    uint32_t raw_len = 4096u + (synth_rand(s) % 28u) * 1024u;
    size_t o = 0;

    while (o < raw_len) {
        size_t run = 1 + synth_rand(s) % 48u;
        uint8_t v = (uint8_t)synth_rand(s);
        if (o + run > raw_len)
            run = raw_len - o;
        memset(raw + o, v, run);
        o += run;
    }

    size_t enc = rle_encode(raw, raw_len, wire + FRAME_HEADER_SIZE,
                            cap - FRAME_HEADER_SIZE - FRAME_TRAILER_SIZE);
    if (enc == 0)
        return 0;

    uint32_t seq = s->next_seq++;
    wire[0] = FRAME_MAGIC0;
    wire[1] = FRAME_MAGIC1;
    wire[2] = FRAME_VERSION;
    wire[3] = (uint8_t)(seq & 3u);
    store_u32(wire + 4, seq);
    store_u32(wire + 8, raw_len);
    store_u32(wire + 12, (uint32_t)enc);

    uint32_t crc = crc32_of(raw, raw_len);
    if (s->damage_every != 0 && seq % s->damage_every == s->damage_every - 1)
        crc ^= 0x00010000u;
    store_u32(wire + FRAME_HEADER_SIZE + enc, crc);
    return FRAME_HEADER_SIZE + enc + FRAME_TRAILER_SIZE;
}
