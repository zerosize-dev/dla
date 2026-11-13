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

static lease_slot make(uint64_t key, uint32_t owner)
{
    lease_slot e;
    memset(&e, 0, sizeof e);
    e.key = key;
    e.owner = owner;
    e.state = SLOT_LIVE;
    e.expires_at = key + 100;
    return e;
}

static void test_put_get(void)
{
    lease_table t;
    CHECK(lease_table_init(&t, 16) == 0);
    for (uint64_t k = 1; k <= 200; k++) {
        lease_slot e = make(k, (uint32_t)k % 7);
        CHECK(lease_table_put(&t, &e) == 0);
    }
    CHECK(lease_table_live(&t) == 200);
    for (uint64_t k = 1; k <= 200; k++) {
        const lease_slot *s = lease_table_get(&t, k);
        CHECK(s != NULL && s->owner == (uint32_t)k % 7);
    }
    CHECK(lease_table_get(&t, 999) == NULL);
    lease_table_destroy(&t);
}

static void test_replace(void)
{
    lease_table t;
    CHECK(lease_table_init(&t, 16) == 0);
    lease_slot a = make(5, 1), b = make(5, 2);
    CHECK(lease_table_put(&t, &a) == 0);
    CHECK(lease_table_put(&t, &b) == 0);
    CHECK(lease_table_live(&t) == 1);
    CHECK(lease_table_get(&t, 5)->owner == 2);
    lease_table_destroy(&t);
}

static void test_remove(void)
{
    lease_table t;
    CHECK(lease_table_init(&t, 16) == 0);
    for (uint64_t k = 1; k <= 40; k++) {
        lease_slot e = make(k, 1);
        lease_table_put(&t, &e);
    }
    for (uint64_t k = 1; k <= 40; k += 2)
        CHECK(lease_table_remove(&t, k) == 0);
    CHECK(lease_table_remove(&t, 1) != 0);
    CHECK(lease_table_live(&t) == 20);
    for (uint64_t k = 1; k <= 40; k++)
        CHECK((lease_table_get(&t, k) != NULL) == (k % 2 == 0));
    lease_table_destroy(&t);
}

static void test_reinsert_after_remove(void)
{
    lease_table t;
    CHECK(lease_table_init(&t, 16) == 0);
    for (int round = 0; round < 50; round++) {
        lease_slot e = make(77, (uint32_t)round);
        CHECK(lease_table_put(&t, &e) == 0);
        CHECK(lease_table_get(&t, 77)->owner == (uint32_t)round);
        CHECK(lease_table_remove(&t, 77) == 0);
    }
    CHECK(lease_table_live(&t) == 0);
    lease_table_destroy(&t);
}

static void test_growth_keeps_entries(void)
{
    lease_table t;
    CHECK(lease_table_init(&t, 64) == 0);
    size_t before = lease_table_capacity(&t);
    for (uint64_t k = 1; k <= 5000; k++) {
        lease_slot e = make(k * 3, 1);
        CHECK(lease_table_put(&t, &e) == 0);
    }
    CHECK(lease_table_capacity(&t) > before);
    for (uint64_t k = 1; k <= 5000; k++)
        CHECK(lease_table_get(&t, k * 3) != NULL);
    lease_table_destroy(&t);
}

static void test_broker_holds_its_window(void)
{
    broker b;
    CHECK(broker_init(&b, 100, 3) == 0);
    for (int i = 0; i < 5000; i++)
        broker_step(&b);
    CHECK(b.granted == 5000);
    CHECK(b.active_len == 100);
    CHECK(lease_table_live(&b.table) == 100);
    for (size_t i = 0; i < b.active_len; i++)
        CHECK(lease_table_get(&b.table, b.active[i]) != NULL);
    broker_destroy(&b);
}

static void test_release_empty(void)
{
    broker b;
    CHECK(broker_init(&b, 4, 1) == 0);
    CHECK(broker_release_one(&b) != 0);
    CHECK(broker_grant(&b, 1, 10) == 0);
    CHECK(broker_release_one(&b) == 0);
    CHECK(b.active_len == 0);
    broker_destroy(&b);
}

int main(void)
{
    test_put_get();
    test_replace();
    test_remove();
    test_reinsert_after_remove();
    test_growth_keeps_entries();
    test_broker_holds_its_window();
    test_release_empty();
    if (failures) {
        fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    puts("all tests passed");
    return 0;
}
