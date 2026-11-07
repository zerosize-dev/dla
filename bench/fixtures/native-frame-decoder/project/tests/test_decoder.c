#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "frame.h"

static int failures;

#define CHECK(cond)                                                          \
    do {                                                                     \
        if (!(cond)) {                                                       \
            fprintf(stderr, "%s:%d: check failed: %s\n", __FILE__, __LINE__, \
                    #cond);                                                  \
            failures++;                                                      \
        }                                                                    \
    } while (0)

static void test_crc(void)
{
    const uint8_t s[] = "123456789";
    CHECK(crc32_of(s, 9) == 0xCBF43926u);
}

static void test_rle_roundtrip(void)
{
    uint8_t raw[600], enc[1300], back[600];
    for (size_t i = 0; i < sizeof raw; i++)
        raw[i] = (uint8_t)(i / 7);
    size_t n = rle_encode(raw, sizeof raw, enc, sizeof enc);
    CHECK(n > 0);
    CHECK(rle_expand(enc, n, back, sizeof back) == 0);
    CHECK(memcmp(raw, back, sizeof raw) == 0);
    CHECK(rle_expand(enc, n - 1, back, sizeof back) != 0);
}

static void test_decode_ok(void)
{
    synth_source s;
    uint8_t *wire = malloc(2u * FRAME_MAX_DECODED + 64u);
    synth_init(&s, 7, 0);
    for (int i = 0; i < 20; i++) {
        size_t n = synth_next(&s, wire, 2u * FRAME_MAX_DECODED + 64u);
        decoded_frame f;
        CHECK(decode_frame(wire, n, &f) == FRAME_OK);
        CHECK(f.seq == (uint32_t)i);
        CHECK(f.len >= 4096);
        frame_release(&f);
        CHECK(f.pixels == NULL);
    }
    free(wire);
}

static void test_decode_rejects(void)
{
    synth_source s;
    uint8_t *wire = malloc(2u * FRAME_MAX_DECODED + 64u);
    decoded_frame f;
    synth_init(&s, 3, 0);
    size_t n = synth_next(&s, wire, 2u * FRAME_MAX_DECODED + 64u);

    CHECK(decode_frame(wire, 10, &f) == FRAME_ERR_SHORT);
    wire[0] = 'X';
    CHECK(decode_frame(wire, n, &f) == FRAME_ERR_MAGIC);
    wire[0] = FRAME_MAGIC0;
    wire[2] = 9;
    CHECK(decode_frame(wire, n, &f) == FRAME_ERR_VERSION);
    wire[2] = FRAME_VERSION;
    CHECK(decode_frame(wire, n - 4, &f) == FRAME_ERR_SIZE);
    free(wire);
}

static void test_damaged_trailer(void)
{
    synth_source s;
    uint8_t *wire = malloc(2u * FRAME_MAX_DECODED + 64u);
    int rejected = 0;
    synth_init(&s, 11, 4);
    for (int i = 0; i < 8; i++) {
        size_t n = synth_next(&s, wire, 2u * FRAME_MAX_DECODED + 64u);
        decoded_frame f;
        frame_status st = decode_frame(wire, n, &f);
        if (st == FRAME_OK)
            frame_release(&f);
        else if (st == FRAME_ERR_CHECKSUM)
            rejected++;
    }
    CHECK(rejected == 2);
    free(wire);
}

int main(void)
{
    test_crc();
    test_rle_roundtrip();
    test_decode_ok();
    test_decode_rejects();
    test_damaged_trailer();
    if (failures == 0)
        puts("all tests passed");
    return failures == 0 ? 0 : 1;
}
