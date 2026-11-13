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

static void test_blocks_are_distinct_and_aligned(void)
{
    void *a = pool_alloc(100), *b = pool_alloc(100), *c = pool_alloc(5);
    CHECK(a && b && c && a != b && b != c);
    CHECK(((uintptr_t)a & 15u) == 0 && ((uintptr_t)b & 15u) == 0 && ((uintptr_t)c & 15u) == 0);
    CHECK(pool_block_size(a) >= 100);
    memset(a, 1, 100);
    memset(b, 2, 100);
    CHECK(((unsigned char *)a)[99] == 1 && ((unsigned char *)b)[0] == 2);
    pool_free(a);
    pool_free(b);
    pool_free(c);
}

static void test_release_makes_room(void)
{
    size_t before = pool_bytes_in_use();
    void *p = pool_alloc(1000);
    CHECK(pool_bytes_in_use() > before);
    pool_free(p);
    CHECK(pool_bytes_in_use() == before);
}

static void test_reuse_of_a_released_block(void)
{
    void *p = pool_alloc(500);
    pool_free(p);
    void *q = pool_alloc(400);
    CHECK(q == p);
    pool_free(q);
}

static void test_oversized_request_is_refused(void)
{
    CHECK(pool_alloc(1u << 20) == NULL);
}

static void test_message_roundtrip(void)
{
    message *m = message_new(7, KIND_BULK, 2000, 99);
    CHECK(m != NULL);
    CHECK(m->len == 2000 && m->id == 7);
    CHECK(message_verify(m) == 0);
    m->payload[10] ^= 0xff;
    CHECK(message_verify(m) != 0);
    message_free(m);
}

static void test_gateway_holds_its_window(void)
{
    gateway g;
    CHECK(gateway_init(&g, 64, 5) == 0);
    for (int i = 0; i < 5000; i++)
        gateway_step(&g);
    CHECK(g.count <= 64);
    CHECK(g.accepted == 5000);
    CHECK(g.accepted - g.acknowledged == g.count);
    for (size_t i = 0; i < g.capacity; i++)
        CHECK(g.inflight[i] == NULL || message_verify(g.inflight[i]) == 0);
    gateway_destroy(&g);
}

static void test_destroy_returns_everything(void)
{
    size_t before = pool_bytes_in_use();
    gateway g;
    CHECK(gateway_init(&g, 32, 3) == 0);
    for (int i = 0; i < 1000; i++)
        gateway_step(&g);
    gateway_destroy(&g);
    CHECK(pool_bytes_in_use() == before);
}

int main(void)
{
    test_blocks_are_distinct_and_aligned();
    test_release_makes_room();
    test_reuse_of_a_released_block();
    test_oversized_request_is_refused();
    test_message_roundtrip();
    test_gateway_holds_its_window();
    test_destroy_returns_everything();
    if (failures) {
        fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    puts("all tests passed");
    return 0;
}
