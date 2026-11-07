#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "glyph.h"

#define RUN_LEN 24
#define HOT_SET 300u
#define REPERTOIRE 2000u

static uint64_t rng_state = 88172645463325252ull;

static uint32_t rng(void)
{
    uint64_t x = rng_state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    rng_state = x;
    return (uint32_t)(x >> 16);
}

/* Most characters come from a small hot set, the rest from the whole
 * repertoire. */
static uint32_t next_codepoint(void)
{
    if (rng() % 10u < 7u)
        return 0x20u + rng() % HOT_SET;
    return 0x20u + rng() % REPERTOIRE;
}

int main(int argc, char **argv)
{
    unsigned long iterations = 2000;
    size_t capacity = 2048;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc)
            iterations = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--capacity") == 0 && i + 1 < argc)
            capacity = strtoul(argv[++i], NULL, 10);
        else {
            fprintf(stderr, "usage: %s [--iterations N] [--capacity C]\n", argv[0]);
            return 2;
        }
    }

    static const uint16_t sizes[] = { 12, 16, 24, 32 };
    glyph_cache *cache = cache_create(capacity);
    if (cache == NULL)
        return 1;

    unsigned long long total_width = 0;
    unsigned long glyphs = 0;

    for (unsigned long i = 0; i < iterations; i++) {
        uint32_t text[RUN_LEN];
        placed_glyph placed[RUN_LEN];
        int width = 0;
        uint16_t face = (uint16_t)(rng() % 3u);
        uint16_t px = sizes[rng() % 4u];

        for (int k = 0; k < RUN_LEN; k++)
            text[k] = next_codepoint();
        glyphs += layout_run(cache, face, px, text, RUN_LEN, placed, RUN_LEN, &width);
        total_width += (unsigned)width;

        if ((i + 1) % 500 == 0 && getenv("TEXTRUN_PROGRESS") != NULL) {
            cache_stats st;
            cache_get_stats(cache, &st);
            fprintf(stderr, "iter=%lu entries=%zu bytes=%zu evictions=%lu\n",
                    i + 1, st.entries, st.bytes, st.evictions);
        }
    }

    cache_stats st;
    cache_get_stats(cache, &st);
    printf("runs=%lu glyphs=%lu width=%llu hits=%lu misses=%lu evictions=%lu "
           "entries=%zu/%zu bytes=%zu\n",
           iterations, glyphs, total_width, st.hits, st.misses, st.evictions,
           st.entries, st.capacity, st.bytes);
    cache_destroy(cache);
    return 0;
}
