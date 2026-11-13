#include <stdlib.h>
#include <string.h>

#include "types.h"

/* Memory comes from malloc in fixed chunks and is cut into blocks with a
 * bump pointer. A released block goes on the front of a single free list. A
 * request walks the first few entries of that list and takes the first block
 * large enough; when the block is bigger than needed, the rest becomes a block
 * of its own. A request nothing fits starts from the chunk. */

#define CHUNK_SIZE (64u * 1024u)
#define GRAIN 16u
#define HEADER 16u
#define MIN_REMAINDER 32u
#define PROBE_LIMIT 16

typedef struct block {
    size_t size;            /* header included */
    struct block *next;     /* valid while the block is free */
} block;

typedef struct chunk {
    struct chunk *next;
} chunk;

static block *free_list;
static chunk *chunks;
static char *bump;
static char *bump_end;
static size_t chunk_count;
static size_t in_use;

static block *header_of(const void *p)
{
    return (block *)((char *)p - HEADER);
}

static void push_free(block *b)
{
    b->next = free_list;
    free_list = b;
}

static block *take_from_list(size_t need)
{
    block **link = &free_list;
    for (int probes = 0; *link != NULL && probes < PROBE_LIMIT; probes++) {
        block *b = *link;
        if (b->size >= need) {
            *link = b->next;
            return b;
        }
        link = &b->next;
    }
    return NULL;
}

static int start_chunk(void)
{
    size_t left = (size_t)(bump_end - bump);
    if (left >= HEADER + MIN_REMAINDER) {
        block *rest = (block *)bump;
        rest->size = left;
        push_free(rest);
    }
    chunk *c = malloc(CHUNK_SIZE);
    if (c == NULL)
        return -1;
    c->next = chunks;
    chunks = c;
    chunk_count++;
    bump = (char *)c + sizeof(chunk) + (GRAIN - sizeof(chunk) % GRAIN) % GRAIN;
    bump_end = (char *)c + CHUNK_SIZE;
    return 0;
}

void *pool_alloc(size_t n)
{
    size_t need = (n + GRAIN - 1) / GRAIN * GRAIN + HEADER;
    if (need > CHUNK_SIZE / 2)
        return NULL;

    block *b = take_from_list(need);
    if (b != NULL) {
        if (b->size >= need + HEADER + MIN_REMAINDER) {
            block *rest = (block *)((char *)b + need);
            rest->size = b->size - need;
            b->size = need;
            push_free(rest);
        }
    } else {
        if ((size_t)(bump_end - bump) < need && start_chunk() != 0)
            return NULL;
        b = (block *)bump;
        b->size = need;
        bump += need;
    }
    in_use += b->size;
    return (char *)b + HEADER;
}

void pool_free(void *p)
{
    if (p == NULL)
        return;
    block *b = header_of(p);
    in_use -= b->size;
    push_free(b);
}

size_t pool_block_size(const void *p)
{
    return header_of(p)->size - HEADER;
}

size_t pool_bytes_in_use(void)
{
    return in_use;
}

size_t pool_chunks(void)
{
    return chunk_count;
}
