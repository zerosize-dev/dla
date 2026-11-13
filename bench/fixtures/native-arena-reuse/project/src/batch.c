#include <stdlib.h>

#include "types.h"

/* A batch keeps its records in memory until it has been totalled. Every
 * SUMMARY_EVERY records a running summary is handed to the carry list. */
#define SUMMARY_EVERY 400u
#define MEMO_MIN 16u
#define MEMO_SPAN 500u

uint64_t settlement_rand(settlement *s)
{
    s->rng ^= s->rng << 13;
    s->rng ^= s->rng >> 7;
    s->rng ^= s->rng << 17;
    return s->rng;
}

void settlement_init(settlement *s, uint64_t seed)
{
    s->rng = seed ? seed : 88172645463325252ULL;
    s->next_id = 1;
    s->batch_no = 0;
    s->records = 0;
    s->total_cents = 0;
    s->digest = 0;
    carry_init(&s->carried);
}

int settlement_run_batch(settlement *s, uint32_t count)
{
    record **rows = calloc(count, sizeof *rows);
    if (rows == NULL)
        return -1;

    s->batch_no++;
    int64_t total = 0;
    uint32_t digest = 0;
    int status = 0;

    for (uint32_t i = 0; i < count; i++) {
        uint64_t r = settlement_rand(s);
        uint32_t memo = MEMO_MIN + (uint32_t)((r >> 8) % MEMO_SPAN);
        int64_t cents = (int64_t)(r % 250000u) - 1000;
        rows[i] = record_new(s->next_id++, cents, memo, r);
        if (rows[i] == NULL) {
            status = -1;
            break;
        }
        total += cents;
        digest ^= record_digest(rows[i]);
        if ((i + 1) % SUMMARY_EVERY == 0)
            carry_add(&s->carried, s->batch_no, i + 1, total, digest);
    }

    for (uint32_t i = 0; i < count; i++)
        record_free(rows[i]);
    free(rows);

    s->records += count;
    s->total_cents += total;
    s->digest ^= digest;
    return status;
}

void settlement_finish(settlement *s)
{
    carry_clear(&s->carried);
}
