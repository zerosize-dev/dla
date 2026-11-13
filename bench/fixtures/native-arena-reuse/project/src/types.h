#ifndef TYPES_H
#define TYPES_H

#include <stddef.h>
#include <stdint.h>

/* The settlement run turns a day's payment records into totals. Every batch
 * reads its records into memory, adds them up and releases them; a short
 * summary of each batch is kept for the next run's reconciliation. */

typedef struct {
    uint32_t id;
    int64_t cents;
    uint32_t memo_len;
    char memo[];
} record;

record *record_new(uint32_t id, int64_t cents, uint32_t memo_len, uint64_t seed);
void record_free(record *r);
uint32_t record_digest(const record *r);

typedef struct {
    uint32_t batch;
    uint32_t count;
    int64_t total_cents;
    uint32_t digest;
} summary;

/* carry.c: the summaries of the most recent batches */
#define CARRY_SLOTS 32

typedef struct {
    summary *slots[CARRY_SLOTS];
    size_t next;
    size_t held;
} carry;

void carry_init(carry *c);
int carry_add(carry *c, uint32_t batch, uint32_t count, int64_t total, uint32_t digest);
size_t carry_held(const carry *c);
void carry_clear(carry *c);

/* batch.c */
typedef struct {
    uint64_t rng;
    uint32_t next_id;
    uint32_t batch_no;
    unsigned long records;
    int64_t total_cents;
    uint32_t digest;
    carry carried;
} settlement;

void settlement_init(settlement *s, uint64_t seed);
int settlement_run_batch(settlement *s, uint32_t count);
void settlement_finish(settlement *s);
uint64_t settlement_rand(settlement *s);

/* report.c */
void report_summary(const settlement *s, unsigned long iterations);
void report_heap(void);

#endif
