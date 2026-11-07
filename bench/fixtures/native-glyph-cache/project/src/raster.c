#include <stdlib.h>
#include <string.h>

#include "glyph.h"

/* Every glyph is described by a few strokes derived from the key; the
 * bitmap is the union of those strokes, anti-aliased by a box filter. */

static uint32_t mix(uint32_t x)
{
    x ^= x >> 16;
    x *= 0x7FEB352Du;
    x ^= x >> 15;
    x *= 0x846CA68Bu;
    x ^= x >> 16;
    return x;
}

int raster_glyph(const glyph_key *key, glyph_bitmap *out)
{
    int px = key->px;
    int w = px / 2 + (int)(mix(key->codepoint) % (unsigned)(px / 2 + 1));
    int h = px;
    if (w < 4)
        w = 4;

    uint8_t *cov = calloc((size_t)w * (size_t)h, 1);
    if (cov == NULL)
        return -1;

    uint32_t seed = mix(key->codepoint ^ ((uint32_t)key->face << 20));
    int strokes = 2 + (int)(seed % 4u);
    for (int s = 0; s < strokes; s++) {
        uint32_t r = mix(seed + (uint32_t)s * 0x9E3779B9u);
        int x0 = (int)(r % (unsigned)w);
        int y0 = (int)((r >> 8) % (unsigned)h);
        int len = 2 + (int)((r >> 16) % (unsigned)(h / 2 + 1));
        int horizontal = (int)((r >> 24) & 1u);
        for (int i = 0; i < len; i++) {
            int x = horizontal ? x0 + i : x0;
            int y = horizontal ? y0 : y0 + i;
            if (x < w && y < h)
                cov[y * w + x] = 255;
        }
    }

    for (int y = 1; y < h - 1; y++) {
        for (int x = 1; x < w - 1; x++) {
            unsigned sum = cov[y * w + x] + cov[y * w + x - 1] +
                           cov[y * w + x + 1];
            if (cov[y * w + x] == 0)
                cov[y * w + x] = (uint8_t)(sum / 6u);
        }
    }

    out->width = w;
    out->height = h;
    out->advance = w + 1 + px / 16;
    out->coverage = cov;
    return 0;
}

void raster_release(glyph_bitmap *bm)
{
    free(bm->coverage);
    memset(bm, 0, sizeof *bm);
}
