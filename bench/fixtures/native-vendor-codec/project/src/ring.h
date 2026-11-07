#ifndef RING_H
#define RING_H

#include <stddef.h>

/* Fixed-capacity ring of item pointers; once full, each new item takes the
 * place of the oldest one. The ring never looks inside the items. */
typedef struct ring ring;

ring *ring_create(size_t slots);
void ring_destroy(ring *r);

void ring_put(ring *r, const void *item);

/* Number of items currently held, and the i-th oldest (NULL when out of range). */
size_t ring_count(const ring *r);
const void *ring_get(const ring *r, size_t i);

#endif
