#include "glyph.h"

size_t layout_run(glyph_cache *c, uint16_t face, uint16_t px,
                  const uint32_t *text, size_t n,
                  placed_glyph *out, size_t out_cap, int *width)
{
    int pen = 0;
    size_t placed = 0;

    for (size_t i = 0; i < n && placed < out_cap; i++) {
        glyph_key key = { face, px, text[i] };
        const glyph_bitmap *bm = cache_lookup(c, &key);
        if (bm == NULL)
            continue;
        out[placed].x = pen;
        out[placed].y = 0;
        out[placed].bitmap = bm;
        placed++;
        pen += bm->advance;
    }

    if (width != NULL)
        *width = pen;
    return placed;
}
