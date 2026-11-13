#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"

/* The router lives as long as the process does. */
static router the_router;

static void usage(const char *argv0)
{
    fprintf(stderr,
            "usage: %s [--iterations N] [--damage-every K] [--unknown-every K] "
            "[--spool PATH] [--seed S]\n",
            argv0);
}

int main(int argc, char **argv)
{
    unsigned long iterations = 20000;
    unsigned long damage_every = 48;
    unsigned long unknown_every = 97;
    unsigned long seed = 1;
    const char *spool_path = "/dev/null";

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc)
            iterations = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--damage-every") == 0 && i + 1 < argc)
            damage_every = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--unknown-every") == 0 && i + 1 < argc)
            unknown_every = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--spool") == 0 && i + 1 < argc)
            spool_path = argv[++i];
        else if (strcmp(argv[i], "--seed") == 0 && i + 1 < argc)
            seed = strtoul(argv[++i], NULL, 10);
        else {
            usage(argv[0]);
            return 2;
        }
    }

    if (router_init(&the_router, spool_path) != 0)
        return 1;

    feed f;
    feed_init(&f, seed, (uint32_t)damage_every, (uint32_t)unknown_every);
    parcel p;
    for (unsigned long i = 0; i < iterations; i++) {
        feed_next(&f, &p);
        router_handle(&the_router, &p);
    }
    report_summary(&the_router, iterations);
    return 0;
}
