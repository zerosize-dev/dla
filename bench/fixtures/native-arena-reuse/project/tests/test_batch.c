#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"

static int failures;

#define CHECK(cond)                                                          \
    do {                                                                     \
        if (!(cond)) {                                                       \
            fprintf(stderr, "%s:%d: check failed: %s\n", __FILE__, __LINE__, \
                    #cond);                                                  \
            failures++;                                                      \
        }                                                                    \
    } while (0)

static void test_record_fields(void)
{
    record *r = record_new(9, -250, 40, 7);
    CHECK(r != NULL);
    CHECK(r->id == 9 && r->cents == -250 && r->memo_len == 40);
    for (uint32_t i = 0; i < r->memo_len; i++)
        CHECK(r->memo[i] >= 'a' && r->memo[i] <= 'z');
    record *q = record_new(9, -250, 40, 7);
    CHECK(record_digest(r) == record_digest(q));
    q->memo[3] = (char)(q->memo[3] == 'a' ? 'b' : 'a');
    CHECK(record_digest(r) != record_digest(q));
    record_free(r);
    record_free(q);
}

static void test_carry_keeps_the_latest(void)
{
    carry c;
    carry_init(&c);
    for (uint32_t i = 1; i <= 100; i++)
        CHECK(carry_add(&c, i, i, (int64_t)i * 10, i) == 0);
    CHECK(carry_held(&c) == CARRY_SLOTS);
    uint32_t oldest = 100;
    for (size_t i = 0; i < CARRY_SLOTS; i++)
        if (c.slots[i]->batch < oldest)
            oldest = c.slots[i]->batch;
    CHECK(oldest == 100 - CARRY_SLOTS + 1);
    carry_clear(&c);
    CHECK(carry_held(&c) == 0);
}

static void test_batch_totals_are_repeatable(void)
{
    settlement a, b;
    settlement_init(&a, 11);
    settlement_init(&b, 11);
    CHECK(settlement_run_batch(&a, 2000) == 0);
    CHECK(settlement_run_batch(&b, 2000) == 0);
    CHECK(a.records == 2000 && a.total_cents == b.total_cents && a.digest == b.digest);
    settlement_finish(&a);
    settlement_finish(&b);
}

static void test_batches_accumulate(void)
{
    settlement s;
    settlement_init(&s, 3);
    for (int i = 0; i < 5; i++)
        CHECK(settlement_run_batch(&s, 1000) == 0);
    CHECK(s.records == 5000 && s.batch_no == 5);
    CHECK(carry_held(&s.carried) == 5 * (1000 / 400));
    settlement_finish(&s);
}

int main(void)
{
    test_record_fields();
    test_carry_keeps_the_latest();
    test_batch_totals_are_repeatable();
    test_batches_accumulate();
    if (failures) {
        fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    puts("all tests passed");
    return 0;
}
