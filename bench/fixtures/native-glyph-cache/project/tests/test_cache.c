#include <stdio.h>
#include <stdlib.h>

#include "glyph.h"

static int failures;

#define CHECK(cond)                                                          \
    do {                                                                     \
        if (!(cond)) {                                                       \
            fprintf(stderr, "%s:%d: check failed: %s\n", __FILE__, __LINE__, \
                    #cond);                                                  \
            failures++;                                                      \
        }                                                                    \
    } while (0)

static void test_raster(void)
{
    glyph_key k = { 1, 16, 'g' };
    glyph_bitmap a, b;
    CHECK(raster_glyph(&k, &a) == 0);
    CHECK(raster_glyph(&k, &b) == 0);
    CHECK(a.width == b.width && a.height == 16);
    int same = 1;
    for (int i = 0; i < a.width * a.height; i++)
        if (a.coverage[i] != b.coverage[i])
            same = 0;
    CHECK(same);
    raster_release(&a);
    raster_release(&b);
    CHECK(a.coverage == NULL);
}

static void test_hit_and_miss(void)
{
    glyph_cache *c = cache_create(4);
    glyph_key k = { 0, 12, 'a' };
    cache_stats st;
    const glyph_bitmap *first = cache_lookup(c, &k);
    const glyph_bitmap *second = cache_lookup(c, &k);
    CHECK(first != NULL && first == second);
    cache_get_stats(c, &st);
    CHECK(st.hits == 1 && st.misses == 1 && st.entries == 1);
    cache_destroy(c);
}

static void test_eviction_order(void)
{
    glyph_cache *c = cache_create(3);
    cache_stats st;
    for (uint32_t cp = 'a'; cp < 'a' + 3; cp++) {
        glyph_key k = { 0, 12, cp };
        CHECK(cache_lookup(c, &k) != NULL);
    }
    glyph_key a = { 0, 12, 'a' };
    glyph_key d = { 0, 12, 'd' };
    cache_lookup(c, &a);          /* 'a' becomes the most recent */
    cache_lookup(c, &d);          /* evicts 'b', the oldest */
    cache_get_stats(c, &st);
    CHECK(st.entries == 3 && st.evictions == 1);

    unsigned long before = st.hits;
    cache_lookup(c, &a);
    cache_get_stats(c, &st);
    CHECK(st.hits == before + 1);

    glyph_key b = { 0, 12, 'b' };
    before = st.misses;
    cache_lookup(c, &b);
    cache_get_stats(c, &st);
    CHECK(st.misses == before + 1);
    cache_destroy(c);
}

static void test_layout(void)
{
    glyph_cache *c = cache_create(16);
    uint32_t text[5] = { 'h', 'e', 'l', 'l', 'o' };
    placed_glyph out[5];
    int width = 0;
    size_t n = layout_run(c, 0, 16, text, 5, out, 5, &width);
    CHECK(n == 5);
    CHECK(out[0].x == 0 && out[1].x == out[0].bitmap->advance);
    CHECK(width > 0);
    CHECK(out[2].bitmap == out[3].bitmap);   /* same glyph, same entry */
    cache_destroy(c);
}

int main(void)
{
    test_raster();
    test_hit_and_miss();
    test_eviction_order();
    test_layout();
    if (failures == 0)
        puts("all tests passed");
    return failures == 0 ? 0 : 1;
}
