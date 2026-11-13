#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"

/* Trace lines name a parcel by a short label. Labels are interned so a line
 * can carry a pointer instead of a string. */

struct label {
    label *next;
    uint32_t hash;
    uint32_t len;
    char text[];
};

static uint32_t hash_text(const char *s, size_t len)
{
    uint32_t h = 2166136261u;
    for (size_t i = 0; i < len; i++) {
        h ^= (uint8_t)s[i];
        h *= 16777619u;
    }
    return h;
}

int label_pool_init(label_pool *p, size_t bucket_count)
{
    p->buckets = calloc(bucket_count, sizeof *p->buckets);
    if (p->buckets == NULL)
        return -1;
    p->bucket_count = bucket_count;
    p->entries = 0;
    return 0;
}

const char *label_intern(label_pool *p, const char *hub, uint16_t lane, uint32_t seq)
{
    char text[64];
    int n = snprintf(text, sizeof text, "hub-%s/lane-%02u/parcel-%08u", hub,
                     (unsigned)lane, (unsigned)seq);
    if (n < 0 || (size_t)n >= sizeof text)
        return NULL;

    uint32_t h = hash_text(text, (size_t)n);
    label **bucket = &p->buckets[h % p->bucket_count];
    for (label *l = *bucket; l != NULL; l = l->next) {
        if (l->hash == h && l->len == (uint32_t)n && memcmp(l->text, text, (size_t)n) == 0)
            return l->text;
    }

    label *l = malloc(sizeof *l + (size_t)n + 1);
    if (l == NULL)
        return NULL;
    l->hash = h;
    l->len = (uint32_t)n;
    memcpy(l->text, text, (size_t)n + 1);
    l->next = *bucket;
    *bucket = l;
    p->entries++;
    return l->text;
}

size_t label_pool_entries(const label_pool *p)
{
    return p->entries;
}
