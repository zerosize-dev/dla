#ifndef TYPES_H
#define TYPES_H

#include <stddef.h>
#include <stdint.h>

/* A lease grants one client the use of one upstream connection slot until it
 * is released or its deadline passes. The broker keeps every active lease in
 * an open-addressing table keyed by the lease id (linear probing). */

typedef enum {
    SLOT_EMPTY = 0,
    SLOT_LIVE = 1,
    SLOT_DELETED = 2
} slot_state;

typedef struct {
    uint64_t key;          /* lease id */
    uint32_t owner;        /* client id */
    uint32_t state;        /* slot_state */
    uint64_t granted_at;   /* logical clock tick */
    uint64_t expires_at;
} lease_slot;

typedef struct {
    lease_slot *slots;
    size_t capacity;
    size_t live;
    size_t deleted;
} lease_table;

/* lease_table.c */
int lease_table_init(lease_table *t, size_t capacity);
void lease_table_destroy(lease_table *t);
int lease_table_put(lease_table *t, const lease_slot *entry);
const lease_slot *lease_table_get(const lease_table *t, uint64_t key);
int lease_table_remove(lease_table *t, uint64_t key);
size_t lease_table_live(const lease_table *t);
size_t lease_table_capacity(const lease_table *t);
uint64_t lease_hash(uint64_t key);

/* broker.c */
typedef struct {
    lease_table table;
    uint64_t *active;      /* ids of the leases currently granted */
    size_t active_len;
    size_t window;         /* leases held at steady state */
    uint64_t next_id;
    uint64_t clock;
    uint64_t rng;
    unsigned long granted;
    unsigned long released;
} broker;

int broker_init(broker *b, size_t window, uint64_t seed);
void broker_destroy(broker *b);
int broker_grant(broker *b, uint32_t owner, uint64_t ttl);
int broker_release_one(broker *b);
void broker_step(broker *b);
uint64_t broker_rand(broker *b);

/* report.c */
void report_summary(const broker *b, unsigned long iterations);

#endif
