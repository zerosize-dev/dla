#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "session.h"

#define PACKET_SIZE 2048

int main(int argc, char **argv)
{
    unsigned long iterations = 2000;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc) {
            iterations = strtoul(argv[++i], NULL, 10);
        } else {
            fprintf(stderr, "usage: %s [--iterations N]\n", argv[0]);
            return 2;
        }
    }

    session s;
    if (session_open(&s, 64, 32) != 0) {
        fprintf(stderr, "cannot open decoder session\n");
        return 1;
    }

    uint8_t packet[PACKET_SIZE];
    uint32_t seed = 12345;
    for (unsigned long i = 0; i < iterations; i++) {
        for (size_t k = 0; k < sizeof packet; k++) {
            seed = seed * 1664525u + 1013904223u;
            packet[k] = (uint8_t)(seed >> 24);
        }
        if (session_feed(&s, packet, sizeof packet) != 0)
            s.errors++;
    }

    const acme_frame *last = sink_replay(&s.out, ring_count(s.out.replay) - 1);
    if (last != NULL)
        printf("last kept frame: %lu bytes\n", (unsigned long)last->size);
    printf("packets=%lu frames=%lu bytes=%llu errors=%lu checksum=%08x\n",
           iterations, s.out.frames, s.out.bytes, s.errors,
           (unsigned)s.out.checksum);
    session_close(&s);
    return 0;
}
