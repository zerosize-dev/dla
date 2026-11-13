#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"

/* The broker lives as long as the process does. */
static broker the_broker;

static void usage(const char *argv0)
{
    fprintf(stderr, "usage: %s [--iterations N] [--window W] [--seed S]\n", argv0);
}

int main(int argc, char **argv)
{
    unsigned long iterations = 200000;
    unsigned long window = 4096;
    unsigned long seed = 1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc)
            iterations = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--window") == 0 && i + 1 < argc)
            window = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--seed") == 0 && i + 1 < argc)
            seed = strtoul(argv[++i], NULL, 10);
        else {
            usage(argv[0]);
            return 2;
        }
    }

    if (broker_init(&the_broker, window, seed) != 0)
        return 1;
    for (unsigned long i = 0; i < iterations; i++)
        broker_step(&the_broker);
    report_summary(&the_broker, iterations);
    return 0;
}
