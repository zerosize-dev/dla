#ifndef RING_H
#define RING_H

#include <stddef.h>
#include <stdint.h>

/* Fixed-size ring of byte records; the oldest record is overwritten when the
 * ring is full. */
typedef struct ring ring;

ring *ring_create(size_t slots, size_t slot_size);
void ring_destroy(ring *r);

/* Copies up to slot_size bytes of `data` into the next slot. */
void ring_put(ring *r, const uint8_t *data, size_t len);

/* Number of records currently held, and the copy of the i-th oldest. */
size_t ring_count(const ring *r);
size_t ring_get(const ring *r, size_t i, uint8_t *out, size_t cap);

#endif
