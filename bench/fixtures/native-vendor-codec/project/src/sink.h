#ifndef SINK_H
#define SINK_H

#include <stddef.h>
#include <stdint.h>

#include "ring.h"

/* Collects decoded frames: keeps the most recent ones in a ring and tracks
 * running totals. */
typedef struct {
    ring *recent;
    unsigned long frames;
    unsigned long long bytes;
    uint32_t checksum;
} sink;

int sink_init(sink *s, size_t keep, size_t frame_cap);
void sink_close(sink *s);

/* Records one frame. The payload is copied into the ring. */
void sink_push(sink *s, const uint8_t *data, size_t len);

#endif
