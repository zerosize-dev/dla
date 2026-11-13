#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"

static settlement the_run;

static void usage(const char *argv0)
{
    fprintf(stderr, "usage: %s [--iterations N] [--batch-size K] [--seed S] [--stats]\n", argv0);
}

int main(int argc, char **argv)
{
    unsigned long iterations = 40;
    unsigned long batch_size = 12000;
    unsigned long seed = 1;
    int stats = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc)
            iterations = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--batch-size") == 0 && i + 1 < argc)
            batch_size = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--seed") == 0 && i + 1 < argc)
            seed = strtoul(argv[++i], NULL, 10);
        else if (strcmp(argv[i], "--stats") == 0)
            stats = 1;
        else {
            usage(argv[0]);
            return 2;
        }
    }
    if (batch_size == 0)
        return 2;

    settlement_init(&the_run, seed);
    for (unsigned long i = 0; i < iterations; i++) {
        /* batches differ a little in size, as the day's feeds do */
        uint32_t n = (uint32_t)(batch_size - batch_size / 8 +
                                settlement_rand(&the_run) % (batch_size / 4 + 1));
        if (settlement_run_batch(&the_run, n) != 0)
            return 1;
    }
    report_summary(&the_run, iterations);
    if (stats)
        report_heap();
    settlement_finish(&the_run);
    return 0;
}
