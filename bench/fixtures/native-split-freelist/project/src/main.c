#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"

/* The relay holds its gateway for the life of the process. */
static gateway the_gateway;

static void usage(const char *argv0)
{
    fprintf(stderr, "usage: %s [--iterations N] [--window W] [--seed S]\n", argv0);
}

int main(int argc, char **argv)
{
    unsigned long iterations = 200000;
    unsigned long window = 2000;
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
    if (window == 0)
        return 2;

    if (gateway_init(&the_gateway, window, seed) != 0)
        return 1;
    for (unsigned long i = 0; i < iterations; i++)
        gateway_step(&the_gateway);
    report_summary(&the_gateway, iterations);
    return 0;
}
