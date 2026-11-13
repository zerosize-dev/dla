#include <stdlib.h>
#include <string.h>

#include "types.h"

#define MIN_CAPACITY 64u

/* Load factor limit: 7 slots in 10 may be in use (live or deleted). */
#define LOAD_NUM 7u
#define LOAD_DEN 10u

uint64_t lease_hash(uint64_t key)
{
    key ^= key >> 30;
    key *= 0xbf58476d1ce4e5b9ULL;
    key ^= key >> 27;
    key *= 0x94d049bb133111ebULL;
    key ^= key >> 31;
    return key;
}

static lease_slot *alloc_slots(size_t capacity)
{
    lease_slot *s = malloc(capacity * sizeof *s);
    if (s == NULL)
        return NULL;
    for (size_t i = 0; i < capacity; i++) {
        s[i].key = 0;
        s[i].owner = 0;
        s[i].state = SLOT_EMPTY;
        s[i].granted_at = 0;
        s[i].expires_at = 0;
    }
    return s;
}

int lease_table_init(lease_table *t, size_t capacity)
{
    size_t cap = capacity < MIN_CAPACITY ? MIN_CAPACITY : capacity;
    t->slots = alloc_slots(cap);
    if (t->slots == NULL)
        return -1;
    t->capacity = cap;
    t->live = 0;
    t->deleted = 0;
    return 0;
}

void lease_table_destroy(lease_table *t)
{
    free(t->slots);
    t->slots = NULL;
    t->capacity = t->live = t->deleted = 0;
}

/* Places an entry known to be absent into a table that has room. */
static void place(lease_table *t, const lease_slot *entry)
{
    size_t i = (size_t)(lease_hash(entry->key) % t->capacity);
    while (t->slots[i].state == SLOT_LIVE)
        i = (i + 1) % t->capacity;
    t->slots[i] = *entry;
    t->slots[i].state = SLOT_LIVE;
}

static int resize(lease_table *t, size_t new_capacity)
{
    lease_slot *fresh = alloc_slots(new_capacity);
    if (fresh == NULL)
        return -1;
    lease_slot *old = t->slots;
    size_t old_capacity = t->capacity;

    t->slots = fresh;
    t->capacity = new_capacity;
    t->deleted = 0;
    for (size_t i = 0; i < old_capacity; i++) {
        if (old[i].state == SLOT_LIVE)
            place(t, &old[i]);
    }
    free(old);
    return 0;
}

int lease_table_put(lease_table *t, const lease_slot *entry)
{
    size_t i = (size_t)(lease_hash(entry->key) % t->capacity);

    for (size_t n = 0; n < t->capacity; n++) {
        lease_slot *s = &t->slots[i];
        if (s->state == SLOT_EMPTY)
            break;
        if (s->state == SLOT_LIVE && s->key == entry->key) {
            *s = *entry;
            s->state = SLOT_LIVE;
            return 0;
        }
        i = (i + 1) % t->capacity;
    }

    /* Slots that are in use, live or deleted, count against the load limit. */
    if ((t->live + t->deleted + 1) * LOAD_DEN > t->capacity * LOAD_NUM) {
        if (resize(t, t->capacity + t->capacity / 4) != 0)
            return -1;
        place(t, entry);
        t->live++;
        return 0;
    }
    t->slots[i] = *entry;
    t->slots[i].state = SLOT_LIVE;
    t->live++;
    return 0;
}

static lease_slot *find(const lease_table *t, uint64_t key)
{
    size_t i = (size_t)(lease_hash(key) % t->capacity);
    for (size_t n = 0; n < t->capacity; n++) {
        lease_slot *s = &t->slots[i];
        if (s->state == SLOT_EMPTY)
            return NULL;
        if (s->state == SLOT_LIVE && s->key == key)
            return s;
        i = (i + 1) % t->capacity;
    }
    return NULL;
}

const lease_slot *lease_table_get(const lease_table *t, uint64_t key)
{
    return find(t, key);
}

int lease_table_remove(lease_table *t, uint64_t key)
{
    lease_slot *s = find(t, key);
    if (s == NULL)
        return -1;
    s->state = SLOT_DELETED;
    t->live--;
    t->deleted++;
    return 0;
}

size_t lease_table_live(const lease_table *t)
{
    return t->live;
}

size_t lease_table_capacity(const lease_table *t)
{
    return t->capacity;
}
