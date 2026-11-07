#include <stdio.h>
#include <string.h>

#include "sink.h"

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
    ring *r = ring_create(3, 8);
    uint8_t out[8];
    for (uint8_t i = 1; i <= 5; i++) {
        uint8_t rec[2] = { i, (uint8_t)(i * 2) };
        ring_put(r, rec, sizeof rec);
    }
    CHECK(ring_count(r) == 3);
    CHECK(ring_get(r, 0, out, sizeof out) == 2 && out[0] == 3);
    CHECK(ring_get(r, 2, out, sizeof out) == 2 && out[0] == 5 && out[1] == 10);
    CHECK(ring_get(r, 3, out, sizeof out) == 0);
    ring_destroy(r);
}

static void test_ring_truncates(void)
{
    ring *r = ring_create(2, 4);
    uint8_t big[10], out[10];
    memset(big, 7, sizeof big);
    ring_put(r, big, sizeof big);
    CHECK(ring_get(r, 0, out, sizeof out) == 4);
    ring_destroy(r);
}

static void test_sink_totals(void)
{
    sink s;
    uint8_t frame[100];
    memset(frame, 9, sizeof frame);
    CHECK(sink_init(&s, 4, 64) == 0);
    for (int i = 0; i < 10; i++)
        sink_push(&s, frame, sizeof frame);
    CHECK(s.frames == 10 && s.bytes == 1000);
    CHECK(ring_count(s.recent) == 4);
    sink_close(&s);
}

int main(void)
{
    test_ring_order();
    test_ring_truncates();
    test_sink_totals();
    if (failures == 0)
        puts("all tests passed");
    return failures == 0 ? 0 : 1;
}
