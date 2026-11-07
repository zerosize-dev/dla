#ifndef SINK_H
#define SINK_H

#include <stddef.h>
#include <stdint.h>

#include <acme/codec.h>

#include "ring.h"

/* Collects decoded frames: keeps the most recent ones for replay and tracks
 * running totals. */
typedef struct {
    ring *replay;
    unsigned long frames;
    unsigned long long bytes;
    uint32_t checksum;
} sink;

int sink_init(sink *s, size_t keep);
void sink_close(sink *s);

/* Records one decoded frame and keeps it for replay. */
void sink_push(sink *s, const acme_frame *f);

/* The i-th oldest frame kept for replay, or NULL. */
const acme_frame *sink_replay(const sink *s, size_t i);

#endif
