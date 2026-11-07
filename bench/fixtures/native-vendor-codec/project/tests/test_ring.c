#include <stdio.h>

#include "ring.h"

static int failures;

#define CHECK(cond)                                                          \
    do {                                                                     \
        if (!(cond)) {                                                       \
            fprintf(stderr, "%s:%d: check failed: %s\n", __FILE__, __LINE__, \
                    #cond);                                                  \
            failures++;                                                      \
        }                                                                    \
    } while (0)

static void test_ring_order(void)
{
    int values[5] = { 1, 2, 3, 4, 5 };
    ring *r = ring_create(3);
    for (int i = 0; i < 5; i++)
        ring_put(r, &values[i]);
    CHECK(ring_count(r) == 3);
    CHECK(*(const int *)ring_get(r, 0) == 3);
    CHECK(*(const int *)ring_get(r, 2) == 5);
    CHECK(ring_get(r, 3) == NULL);
    ring_destroy(r);
}

static void test_ring_partial(void)
{
    int a = 7;
    ring *r = ring_create(4);
    CHECK(ring_count(r) == 0);
    CHECK(ring_get(r, 0) == NULL);
    ring_put(r, &a);
    CHECK(ring_count(r) == 1);
    CHECK(ring_get(r, 0) == &a);
    ring_destroy(r);
}

int main(void)
{
    test_ring_order();
    test_ring_partial();
    if (failures == 0)
        puts("all tests passed");
    return failures == 0 ? 0 : 1;
}
