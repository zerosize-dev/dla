#ifndef GLYPH_H
#define GLYPH_H

#include <stddef.h>
#include <stdint.h>

/* Identifies one rendered glyph: which face, at which pixel size. */
typedef struct {
    uint16_t face;
    uint16_t px;
    uint32_t codepoint;
} glyph_key;

typedef struct {
    int width;
    int height;
    int advance;
    uint8_t *coverage;   /* width * height bytes, row-major */
} glyph_bitmap;

/* raster.c: draws a glyph from a synthetic outline description. */
int raster_glyph(const glyph_key *key, glyph_bitmap *out);
void raster_release(glyph_bitmap *bm);

/* cache.c: least-recently-used cache of rendered glyphs, holding at most
 * `capacity` entries. */
typedef struct glyph_cache glyph_cache;

typedef struct {
    size_t entries;
    size_t capacity;
    unsigned long hits;
    unsigned long misses;
    unsigned long evictions;
    size_t bytes;
} cache_stats;

glyph_cache *cache_create(size_t capacity);
void cache_destroy(glyph_cache *c);
const glyph_bitmap *cache_lookup(glyph_cache *c, const glyph_key *key);
void cache_get_stats(const glyph_cache *c, cache_stats *out);

/* layout.c: places a run of codepoints on a line. */
typedef struct {
    int x;
    int y;
    const glyph_bitmap *bitmap;
} placed_glyph;

size_t layout_run(glyph_cache *c, uint16_t face, uint16_t px,
                  const uint32_t *text, size_t n,
                  placed_glyph *out, size_t out_cap, int *width);

#endif
