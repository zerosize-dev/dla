#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "frame.h"

#define WIRE_CAP (2u * FRAME_MAX_DECODED + 64u)

static void usage(const char *argv0)
{
    fprintf(stderr, "usage: %s [--iterations N] [--damage-every K] [--seed S]\n", argv0);
}

int main(int argc, char **argv)
{
    unsigned long iterations = 2000;
    unsigned long damage_every = 16;
    unsigned long seed = 1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc)
            iterations = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--damage-every") == 0 && i + 1 < argc)
            damage_every = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--seed") == 0 && i + 1 < argc)
            seed = strtoul(argv[++i], NULL, 10);
        else {
            usage(argv[0]);
            return 2;
        }
    }

    uint8_t *wire = malloc(WIRE_CAP);
    if (wire == NULL)
        return 1;

    synth_source src;
    synth_init(&src, seed, (uint32_t)damage_every);

    unsigned long decoded = 0, rejected = 0;
    unsigned long long bytes = 0;
    uint32_t digest = 0;

    for (unsigned long i = 0; i < iterations; i++) {
        size_t n = synth_next(&src, wire, WIRE_CAP);
        decoded_frame f;
        frame_status st = decode_frame(wire, n, &f);
        if (st != FRAME_OK) {
            rejected++;
            continue;
        }
        digest = crc32_update(digest, f.pixels, f.len > 64 ? 64 : f.len);
        bytes += f.len;
        decoded++;
        frame_release(&f);
    }

    printf("frames=%lu decoded=%lu rejected=%lu bytes=%llu digest=%08x\n",
           iterations, decoded, rejected, bytes, (unsigned)digest);
    free(wire);
    return 0;
}
