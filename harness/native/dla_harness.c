/*
 * DLA loop driver for C projects. One process, one "checkpoint" record per round (x = units
 * completed), appended to the file named by DLA_HARNESS_OUT.
 *
 * Fill the two DLA-HOOK bodies below and nothing else; keep the loop as it is.
 *
 * Keys: DLA_KEY_MODE=fresh uses key = i (every unit sees a new key); DLA_KEY_MODE=repeated uses
 * key = i % repeat (a bounded key space). Rounds: DLA_WARMUP warm-up rounds, then DLA_ROUNDS
 * measured rounds of DLA_STEP units each (defaults are baked in at generation time).
 *
 * Build from the scratch project root, compiling the project sources (not their main) with it:
 *     cc -std=gnu11 -O2 -g -Isrc -o dla_harness <harness dir>/dla_harness.c <project sources>
 * Run:
 *     DLA_HARNESS_OUT=<harness dir>/out/0.jsonl ./dla_harness
 * The driver reads the glibc allocator itself (mallinfo2, malloc_trim). Do not preload
 * libdlashim.so into it unless DLA_SHIM_DISABLE=1 is set; to get call sites for a driver run,
 * preload the shim WITHOUT the driver's records by using command mode instead.
 */
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#ifdef __GLIBC__
#include <malloc.h>
#endif

enum { DEFAULT_STEP = @@DLA_STEP@@, DEFAULT_WARMUP = @@DLA_WARMUP@@, DEFAULT_ROUNDS = @@DLA_ROUNDS@@,
       DEFAULT_REPEAT = @@DLA_REPEAT@@ };

/* DLA-HOOK state: declare the file-scope variables setup() fills and unit() uses (handles to the services, caches, pools under test) */

static void setup(void)
{
    /* DLA-HOOK setup: construct the objects under test once, before the first round; keep them in the file-scope variables declared above */
    fprintf(stderr, "DLA-HOOK setup has not been filled in\n");
    abort();
}

static void unit(unsigned long i, unsigned long key)
{
    /* DLA-HOOK unit: perform exactly one unit of the workload for key (derive ids/names/payloads from key so a repeated key reproduces an earlier request); i is the 1-based unit number */
    (void)i;
    (void)key;
    fprintf(stderr, "DLA-HOOK unit has not been filled in\n");
    abort();
}

/* ---- nothing below this line is edited ---------------------------------------------------- */

static unsigned long env_ul(const char *name, unsigned long dflt)
{
    const char *v = getenv(name);
    return (v != NULL && *v != 0) ? strtoul(v, NULL, 10) : dflt;
}

typedef struct {
    int64_t rss, rss_anon, rss_file, rss_shmem, swap, vm_data, vm_hwm, threads;
} osv_t;

static int64_t status_kb(const char *buf, const char *key)
{
    size_t kl = strlen(key);
    for (const char *p = buf; *p;) {
        if (strncmp(p, key, kl) == 0) {
            p += kl;
            while (*p == ' ' || *p == '\t')
                p++;
            return strtoll(p, NULL, 10);
        }
        while (*p && *p != '\n')
            p++;
        if (*p)
            p++;
    }
    return -1;
}

static void read_os(osv_t *o)
{
    char buf[8192];
    memset(o, 0xff, sizeof *o); /* all -1: unreadable */
    FILE *f = fopen("/proc/self/status", "r");
    if (f == NULL)
        return;
    size_t n = fread(buf, 1, sizeof buf - 1, f);
    fclose(f);
    buf[n] = 0;
    int64_t kb;
    o->rss = (kb = status_kb(buf, "VmRSS:")) < 0 ? -1 : kb * 1024;
    o->rss_anon = (kb = status_kb(buf, "RssAnon:")) < 0 ? -1 : kb * 1024;
    o->rss_file = (kb = status_kb(buf, "RssFile:")) < 0 ? -1 : kb * 1024;
    o->rss_shmem = (kb = status_kb(buf, "RssShmem:")) < 0 ? -1 : kb * 1024;
    o->swap = (kb = status_kb(buf, "VmSwap:")) < 0 ? -1 : kb * 1024;
    o->vm_data = (kb = status_kb(buf, "VmData:")) < 0 ? -1 : kb * 1024;
    o->vm_hwm = (kb = status_kb(buf, "VmHWM:")) < 0 ? -1 : kb * 1024;
    o->threads = status_kb(buf, "Threads:");
}

static void kv(FILE *f, int *first, const char *k, int64_t v)
{
    if (v < 0)
        return;
    fprintf(f, "%s\"%s\":%lld", *first ? "" : ",", k, (long long)v);
    *first = 0;
}

static void put_os(FILE *f, const osv_t *o, int *first)
{
    kv(f, first, "os.rss", o->rss);
    kv(f, first, "os.rss_anon", o->rss_anon);
    kv(f, first, "os.rss_file", o->rss_file);
    kv(f, first, "os.rss_shmem", o->rss_shmem);
    kv(f, first, "os.swap", o->swap);
    kv(f, first, "os.vm_data", o->vm_data);
    kv(f, first, "os.vm_hwm", o->vm_hwm);
    kv(f, first, "os.threads", o->threads);
}

static void json_str(FILE *f, const char *s)
{
    fputc('"', f);
    for (; *s; s++) {
        unsigned char c = (unsigned char)*s;
        if (c == '"' || c == '\\')
            fprintf(f, "\\%c", c);
        else if (c < 0x20)
            fprintf(f, "\\u%04x", c);
        else
            fputc(c, f);
    }
    fputc('"', f);
}

static struct timespec t_start;

static void checkpoint(unsigned long done, int last, char **argv)
{
    const char *path = getenv("DLA_HARNESS_OUT");
    if (path == NULL || *path == 0)
        return;
    FILE *f = fopen(path, "a");
    if (f == NULL)
        return;
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    long long t_ms = (long long)(now.tv_sec - t_start.tv_sec) * 1000 + (now.tv_nsec - t_start.tv_nsec) / 1000000;
    osv_t os;
    read_os(&os);
    fprintf(f, "{\"dla_harness\":1,\"phase\":\"checkpoint\",\"pid\":%ld,\"argv\":[", (long)getpid());
    json_str(f, argv[0]);
    fprintf(f, "],\"exe\":");
    json_str(f, argv[0]);
    fprintf(f, ",\"runtime\":\"native\",\"x\":%lu,\"t_ms\":%lld,\"gc_forced\":false,\"values\":{", done, t_ms);
    int first = 1;
#if defined(__GLIBC__) && __GLIBC_PREREQ(2, 33)
    struct mallinfo2 mi = mallinfo2();
    /* uordblks: bytes in use in the arenas; hblkhd: bytes in mmap'ed blocks; fordblks: free bytes
     * the allocator holds (the headroom a trim can return); arena: bytes obtained from the OS. */
    kv(f, &first, "native.live_bytes", (int64_t)(mi.uordblks + mi.hblkhd));
    kv(f, &first, "native.free_bytes", (int64_t)mi.fordblks);
    kv(f, &first, "native.arena_bytes", (int64_t)(mi.arena + mi.hblkhd));
#endif
    put_os(f, &os, &first);
    fputc('}', f);
    const char *kind = getenv("DLA_PURGE");
    const char *lim = NULL;
    if (last && kind != NULL && *kind && strcmp(kind, "none") != 0) {
        if (strcmp(kind, "trim") == 0) {
#ifdef __GLIBC__
            osv_t after;
            int rc = malloc_trim(0);
            read_os(&after);
            fprintf(f, ",\"purge\":{\"kind\":\"trim\",\"before\":{");
            first = 1;
            put_os(f, &os, &first);
            fprintf(f, "},\"after\":{");
            first = 1;
            put_os(f, &after, &first);
            fprintf(f, "},\"detail\":\"malloc_trim(0) -> %d\"}", rc);
#else
            lim = "purge trim needs glibc malloc_trim";
#endif
        } else {
            lim = "purge kind is not supported by the c driver (supported: trim)";
        }
    }
    fprintf(f, ",\"groups\":{},\"limitations\":[");
#if !(defined(__GLIBC__) && __GLIBC_PREREQ(2, 33))
    fprintf(f, "\"mallinfo2 is unavailable: only os.* values are recorded\"%s", lim ? "," : "");
#endif
    if (lim != NULL)
        json_str(f, lim);
    fprintf(f, "]}\n");
    fclose(f);
}

int main(int argc, char **argv)
{
    (void)argc;
    unsigned long step = env_ul("DLA_STEP", DEFAULT_STEP);
    unsigned long warmup = env_ul("DLA_WARMUP", DEFAULT_WARMUP);
    unsigned long rounds = env_ul("DLA_ROUNDS", DEFAULT_ROUNDS);
    unsigned long repeat = env_ul("DLA_REPEAT", DEFAULT_REPEAT);
    if (repeat == 0)
        repeat = 1;
    const char *km = getenv("DLA_KEY_MODE");
    int repeated = km != NULL && strcmp(km, "repeated") == 0;
    clock_gettime(CLOCK_MONOTONIC, &t_start);
    setup();
    unsigned long total = warmup + rounds, done = 0;
    for (unsigned long round = 1; round <= total; round++) {
        for (unsigned long k = 0; k < step; k++) {
            done++;
            unit(done, repeated ? done % repeat : done);
        }
        checkpoint(done, round == total, argv);
    }
    return 0;
}
