#include "frame.h"

/* Payload encoding: a sequence of (count, value) pairs, count in 1..255. */

size_t rle_encode(const uint8_t *in, size_t n, uint8_t *out, size_t cap)
{
    size_t o = 0;
    size_t i = 0;
    while (i < n) {
        uint8_t v = in[i];
        size_t run = 1;
        while (i + run < n && in[i + run] == v && run < 255)
            run++;
        if (o + 2 > cap)
            return 0;
        out[o++] = (uint8_t)run;
        out[o++] = v;
        i += run;
    }
    return o;
}

int rle_expand(const uint8_t *in, size_t n, uint8_t *out, size_t want)
{
    size_t o = 0;
    if (n % 2 != 0)
        return -1;
    for (size_t i = 0; i < n; i += 2) {
        size_t run = in[i];
        if (run == 0 || o + run > want)
            return -1;
        for (size_t k = 0; k < run; k++)
            out[o++] = in[i + 1];
    }
    return o == want ? 0 : -1;
}
