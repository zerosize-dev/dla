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

static void test_manifest_sum(void)
{
    const uint8_t s[] = "Wikipedia";
    CHECK(manifest_sum(s, 9) == 0x11E60398u);
    CHECK(manifest_sum(s, 0) == 1u);
}

static void test_feed_is_deterministic(void)
{
    feed a, b;
    parcel pa, pb;
    feed_init(&a, 5, 0, 0);
    feed_init(&b, 5, 0, 0);
    for (int i = 0; i < 50; i++) {
        feed_next(&a, &pa);
        feed_next(&b, &pb);
        CHECK(memcmp(&pa, &pb, sizeof pa) == 0);
        CHECK(pa.seq == (uint32_t)i + 1);
        CHECK(manifest_sum(pa.manifest, pa.manifest_len) == pa.sum);
    }
}

static void test_spool_roundtrip(void)
{
    spool *s = spool_open("/dev/null");
    CHECK(s != NULL);
    char big[10000];
    memset(big, 'x', sizeof big);
    CHECK(spool_write(s, big, sizeof big) == 0);
    spool_close(s);
    CHECK(spool_open("/nonexistent-dir/spool") == NULL);
}

static void test_intern_is_stable(void)
{
    label_pool p;
    CHECK(label_pool_init(&p, 64) == 0);
    const char *a = label_intern(&p, "AMS3", 2, 17);
    const char *b = label_intern(&p, "AMS3", 2, 17);
    const char *c = label_intern(&p, "AMS3", 2, 18);
    CHECK(a != NULL && a == b && a != c);
    CHECK(label_pool_entries(&p) == 2);
    CHECK(strcmp(a, "hub-AMS3/lane-02/parcel-00000017") == 0);
}

static void test_routing_outcomes(void)
{
    router r;
    CHECK(router_init(&r, "/dev/null") == 0);
    feed f;
    parcel p;

    feed_init(&f, 9, 0, 0);
    for (int i = 0; i < 100; i++) {
        feed_next(&f, &p);
        CHECK(router_handle(&r, &p) == ROUTE_OK);
    }
    CHECK(r.routed == 100 && r.rejected == 0);

    feed_next(&f, &p);
    strcpy(p.hub, "ZZZ9");
    CHECK(router_handle(&r, &p) == ROUTE_ERR_HUB);

    feed_next(&f, &p);
    p.manifest[3] ^= 1;
    CHECK(router_handle(&r, &p) == ROUTE_ERR_CHECKSUM);
    CHECK(r.rejected == 2);

    unsigned long total = 0;
    for (unsigned i = 0; i < HUB_COUNT; i++)
        total += r.per_hub[i];
    CHECK(total == r.routed);
}

static void test_spool_failure_is_reported(void)
{
    router r;
    CHECK(router_init(&r, "/nonexistent-dir/spool") == 0);
    feed f;
    parcel p;
    feed_init(&f, 2, 0, 0);
    feed_next(&f, &p);
    CHECK(router_handle(&r, &p) == ROUTE_ERR_SPOOL);
    CHECK(r.routed == 0);
}

int main(void)
{
    test_manifest_sum();
    test_feed_is_deterministic();
    test_spool_roundtrip();
    test_intern_is_stable();
    test_routing_outcomes();
    test_spool_failure_is_reported();
    if (failures) {
        fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    puts("all tests passed");
    return 0;
}
