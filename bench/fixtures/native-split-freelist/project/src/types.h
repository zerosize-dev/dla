#ifndef TYPES_H
#define TYPES_H

#include <stddef.h>
#include <stdint.h>

/* The relay sits between field devices and the archive. Devices send short
 * status frames and larger bulk records; each one is held in a pool buffer
 * until the archive acknowledges it. */

/* pool.c: size-ordered buffer pool carved out of fixed chunks. */
void *pool_alloc(size_t n);
void pool_free(void *p);
size_t pool_block_size(const void *p);
size_t pool_bytes_in_use(void);
size_t pool_chunks(void);

/* message.c */
typedef struct {
    uint32_t id;
    uint16_t kind;          /* 0 status, 1 bulk */
    uint16_t flags;
    uint32_t len;
    uint32_t sum;
    uint8_t payload[];
} message;

enum { KIND_STATUS = 0, KIND_BULK = 1 };

message *message_new(uint32_t id, uint16_t kind, uint32_t len, uint64_t seed);
void message_free(message *m);
int message_verify(const message *m);

/* gateway.c */
typedef struct {
    message **inflight;
    size_t capacity;        /* messages awaiting an acknowledgement */
    size_t count;
    uint64_t rng;
    uint32_t next_id;
    unsigned long accepted;
    unsigned long acknowledged;
    unsigned long bulk;
} gateway;

int gateway_init(gateway *g, size_t capacity, uint64_t seed);
void gateway_destroy(gateway *g);
void gateway_step(gateway *g);
uint64_t gateway_rand(gateway *g);

/* report.c */
void report_summary(const gateway *g, unsigned long iterations);

#endif
