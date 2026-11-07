#include <stdlib.h>
#include <string.h>

#include "glyph.h"

typedef struct entry {
    glyph_key key;
    glyph_bitmap bitmap;
    struct entry *chain;      /* next entry in the same hash bucket */
    struct entry *newer;      /* recency list, towards the head */
    struct entry *older;      /* recency list, towards the tail */
} entry;

struct glyph_cache {
    entry **buckets;
    size_t nbuckets;
    entry *head;              /* most recently used */
    entry *tail;              /* least recently used */
    size_t capacity;
    size_t count;
    size_t bytes;
    unsigned long hits;
    unsigned long misses;
    unsigned long evictions;
};

static size_t hash_key(const glyph_key *k)
{
    uint64_t h = 1469598103934665603ull;
    h = (h ^ k->face) * 1099511628211ull;
    h = (h ^ k->px) * 1099511628211ull;
    h = (h ^ k->codepoint) * 1099511628211ull;
    return (size_t)(h ^ (h >> 29));
}

static int same_key(const glyph_key *a, const glyph_key *b)
{
    return a->face == b->face && a->px == b->px && a->codepoint == b->codepoint;
}

glyph_cache *cache_create(size_t capacity)
{
    glyph_cache *c = calloc(1, sizeof *c);
    if (c == NULL)
        return NULL;
    c->capacity = capacity;
    c->nbuckets = capacity * 2 + 1;
    c->buckets = calloc(c->nbuckets, sizeof *c->buckets);
    if (c->buckets == NULL) {
        free(c);
        return NULL;
    }
    return c;
}

static void list_unlink(glyph_cache *c, entry *e)
{
    if (e->newer)
        e->newer->older = e->older;
    else
        c->head = e->older;
    if (e->older)
        e->older->newer = e->newer;
    else
        c->tail = e->newer;
    e->newer = e->older = NULL;
}

static void list_push_front(glyph_cache *c, entry *e)
{
    e->newer = NULL;
    e->older = c->head;
    if (c->head)
        c->head->newer = e;
    c->head = e;
    if (c->tail == NULL)
        c->tail = e;
}

static void bucket_remove(glyph_cache *c, entry *e)
{
    entry **link = &c->buckets[hash_key(&e->key) % c->nbuckets];
    while (*link != NULL && *link != e)
        link = &(*link)->chain;
    if (*link == e)
        *link = e->chain;
}

static void evict_oldest(glyph_cache *c)
{
    entry *victim = c->tail;
    if (victim == NULL)
        return;
    list_unlink(c, victim);
    bucket_remove(c, victim);
    c->bytes -= (size_t)victim->bitmap.width * (size_t)victim->bitmap.height;
    c->count--;
    c->evictions++;
    raster_release(&victim->bitmap);
    free(victim);
}

const glyph_bitmap *cache_lookup(glyph_cache *c, const glyph_key *key)
{
    size_t b = hash_key(key) % c->nbuckets;
    for (entry *e = c->buckets[b]; e != NULL; e = e->chain) {
        if (same_key(&e->key, key)) {
            c->hits++;
            if (c->head != e) {
                list_unlink(c, e);
                list_push_front(c, e);
            }
            return &e->bitmap;
        }
    }

    c->misses++;
    entry *e = calloc(1, sizeof *e);
    if (e == NULL)
        return NULL;
    e->key = *key;
    if (raster_glyph(key, &e->bitmap) != 0) {
        free(e);
        return NULL;
    }

    while (c->count >= c->capacity)
        evict_oldest(c);

    e->chain = c->buckets[b];
    c->buckets[b] = e;
    list_push_front(c, e);
    c->count++;
    c->bytes += (size_t)e->bitmap.width * (size_t)e->bitmap.height;
    return &e->bitmap;
}

void cache_get_stats(const glyph_cache *c, cache_stats *out)
{
    out->entries = c->count;
    out->capacity = c->capacity;
    out->hits = c->hits;
    out->misses = c->misses;
    out->evictions = c->evictions;
    out->bytes = c->bytes;
}

void cache_destroy(glyph_cache *c)
{
    while (c->tail != NULL)
        evict_oldest(c);
    free(c->buckets);
    free(c);
}
