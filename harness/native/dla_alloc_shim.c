/*
 * DLA allocator shim: an LD_PRELOAD library (Linux) or DYLD_INSERT_LIBRARIES dylib (macOS) that
 * interposes the C allocator, keeps live bytes / blocks / peak, aggregates live memory per
 * allocation call stack, and writes one JSON line to DLA_HARNESS_OUT when the process exits.
 *
 * Build (Linux):  cc -std=gnu11 -O2 -g -fPIC -shared -fno-optimize-sibling-calls \
 *                    -o libdlashim.so dla_alloc_shim.c -ldl -lpthread
 * Build (macOS):  cc -std=gnu11 -O2 -g -dynamiclib -o libdlashim.dylib dla_alloc_shim.c
 * Use:            LD_PRELOAD=$PWD/libdlashim.so DLA_HARNESS_OUT=out.jsonl ./program ...
 *
 * Environment: DLA_HARNESS_OUT (required, else the shim stays inert), DLA_PURGE (trim|jemalloc|
 * none; anything else becomes a limitation), DLA_PROJECT (unused here), DLA_ITER (copied to "x"),
 * DLA_SHIM_ONLY=<basename> (only that executable is measured), DLA_SHIM_SAMPLE=k (capture the call
 * stack of one allocation in k; totals stay exact), DLA_SHIM_SIGTERM=0 (do not dump on SIGTERM),
 * DLA_SHIM_DISABLE=1 (stay inert; set when a driver in the same process writes the records).
 *
 * Design: every tracked block is recorded in one lock-protected open-addressing hash table
 * pointer -> (requested size, site). A per-block header was rejected: it would change the
 * alignment contract of posix_memalign/aligned_alloc, break free() of blocks that were allocated
 * before the shim resolved the real allocator, and break malloc_usable_size(). The table lives in
 * mmap'd memory (never in malloc) and is guarded by a spin lock; a block is removed from the table
 * BEFORE it is handed to the real free/realloc, so another thread that is given the same address
 * cannot collide with a stale entry (see track_free and my_realloc). Sites are aggregated in a
 * fixed table of 4096 stacks keyed by the first 8 return addresses; further stacks are counted in
 * an overflow bucket. Stacks are NOT symbolized here: the dump carries raw return addresses plus
 * the module (path, load bias) so that `addr2line -e <path> -f -C -i <off>` works for PIE
 * executables and shared objects. "o" is the offset of (return address - 1), the call instruction.
 *
 * Limits (also in README.md): fork children do not dump (exec'd children do, to the same file
 * with their own pid); statically linked executables and runtimes with their own allocator (Go,
 * a statically linked jemalloc/tcmalloc build) are not covered; the global spin lock serialises
 * allocation-heavy multithreaded programs (use DLA_SHIM_SAMPLE for cheaper stacks); the recorded
 * size is the requested size, not the usable size; allocations made through the shim's guard
 * (backtrace internals, dump) are not counted; SIGKILL and _exit() leave no dump.
 */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <execinfo.h>
#include <fcntl.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>
#include <pthread.h>
#include <dirent.h>
#include <sched.h>
#ifdef __linux__
#include <link.h>
#endif
#ifdef __GLIBC__
#include <malloc.h>
#endif
#ifdef __APPLE__
#include <malloc/malloc.h>
#endif

#define DLA_EXPORT __attribute__((visibility("default")))
#define DLA_TLS __thread __attribute__((tls_model("initial-exec")))

enum {
    STACK_DEPTH = 12,    /* frames captured per allocation */
    KEEP_FRAMES = 8,     /* return addresses stored per stack */
    SKIP_FRAMES = 2,     /* record_site and the interposed entry point */
    MAX_SITES = 4096,    /* fixed aggregation table */
    SITE_IDX = 8192,     /* open-addressing index over the site table */
    MAX_DUMP_SITES = 1024,
    MAX_MODULES = 256,
    ARENA_BYTES = 1 << 17,
};
#define SITE_NONE 0xFFFFu      /* stack table full */
#define SITE_UNSAMPLED 0xFFFEu /* skipped by DLA_SHIM_SAMPLE */
#define SIZE_MASK ((UINT64_C(1) << 48) - 1)

/* ---------------------------------------------------------------------------------------- */
/* state                                                                                     */

static void *(*real_malloc)(size_t);
static void (*real_free)(void *);
static void *(*real_calloc)(size_t, size_t);
static void *(*real_realloc)(void *, size_t);
static int (*real_posix_memalign)(void **, size_t, size_t);

enum { ST_UNINIT = 0, ST_RESOLVING = 1, ST_READY = 2 };
static _Atomic int resolve_state;
static int active; /* 1 when the shim tracks this process */
static int sample_every = 1;
static int sigterm_enabled = 1;
static pid_t init_pid;
static struct timespec t0;
static DLA_TLS int in_shim;
static DLA_TLS unsigned long sample_tick;

static _Atomic int64_t live_bytes, live_blocks, peak_bytes, total_allocs;
static _Atomic int64_t untracked_allocs; /* hash table could not grow */
static _Atomic int64_t stale_replaced;   /* insert found a stale entry (freed by an unseen path) */
static _Atomic int64_t unknown_frees;    /* free/realloc of a pointer the table does not know */

/* bootstrap arena for allocations made while dlsym resolves the real allocator */
static _Alignas(16) unsigned char arena[ARENA_BYTES];
static _Atomic size_t arena_top;

static int in_arena(const void *p)
{
    return (const unsigned char *)p >= arena && (const unsigned char *)p < arena + ARENA_BYTES;
}

static void *arena_alloc(size_t n)
{
    size_t need = (n + 15) & ~(size_t)15;
    size_t off = atomic_fetch_add(&arena_top, need + 16);
    if (off + need + 16 > ARENA_BYTES) {
        errno = ENOMEM;
        return NULL;
    }
    *(size_t *)(arena + off) = n; /* header: requested size, for realloc */
    return arena + off + 16;      /* static storage is already zeroed */
}

/* ---------------------------------------------------------------------------------------- */
/* spin lock                                                                                 */

typedef struct {
    atomic_flag flag;
} spin_t;

static void spin_lock(spin_t *l)
{
    unsigned spins = 0;
    while (atomic_flag_test_and_set_explicit(&l->flag, memory_order_acquire)) {
        if (++spins > 64) {
            sched_yield();
            spins = 0;
        }
    }
}

static void spin_unlock(spin_t *l) { atomic_flag_clear_explicit(&l->flag, memory_order_release); }

/* ---------------------------------------------------------------------------------------- */
/* pointer table: ptr -> (size | site << 48), sharded by pointer hash                        */

typedef struct {
    uintptr_t ptr; /* 0 = empty */
    uint64_t meta;
} ent_t;

enum { SHARDS = 64, SHARD_INIT = 1 << 10 };

typedef struct {
    spin_t lock;
    ent_t *tab;
    size_t cap; /* power of two */
    size_t count;
} shard_t;

static shard_t shards[SHARDS];
static _Atomic int tab_broken; /* mmap failed for a shard: tracking is unreliable */

static uint64_t mix(uintptr_t p) { return ((uint64_t)(p >> 4) * UINT64_C(0x9E3779B97F4A7C15)); }
static shard_t *shard_of(uintptr_t p) { return &shards[mix(p) >> 58]; }
static size_t slot_of(uintptr_t p, size_t cap) { return (size_t)((mix(p) >> 12) & (cap - 1)); }

static ent_t *map_table(size_t cap)
{
    void *m = mmap(NULL, cap * sizeof(ent_t), PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    return m == MAP_FAILED ? NULL : (ent_t *)m;
}

static void raw_insert(ent_t *t, size_t cap, uintptr_t p, uint64_t meta)
{
    size_t i = slot_of(p, cap);
    while (t[i].ptr != 0)
        i = (i + 1) & (cap - 1);
    t[i].ptr = p;
    t[i].meta = meta;
}

/* Shard lock held. Returns 0 when not stored, 1 when stored, 2 when a stale entry was replaced. */
static int tab_insert(shard_t *sh, uintptr_t p, uint64_t meta, uint64_t *old_meta)
{
    if (sh->tab == NULL) {
        sh->tab = map_table(SHARD_INIT);
        if (sh->tab == NULL) {
            atomic_store(&tab_broken, 1);
            return 0;
        }
        sh->cap = SHARD_INIT;
    }
    if ((sh->count + 1) * 2 > sh->cap) {
        ent_t *nt = sh->cap < ((size_t)1 << 28) ? map_table(sh->cap * 2) : NULL;
        if (nt != NULL) {
            for (size_t i = 0; i < sh->cap; i++)
                if (sh->tab[i].ptr != 0)
                    raw_insert(nt, sh->cap * 2, sh->tab[i].ptr, sh->tab[i].meta);
            munmap(sh->tab, sh->cap * sizeof(ent_t));
            sh->tab = nt;
            sh->cap *= 2;
        } else if ((sh->count + 1) * 10 > sh->cap * 9) {
            return 0;
        }
    }
    size_t i = slot_of(p, sh->cap);
    while (sh->tab[i].ptr != 0) {
        if (sh->tab[i].ptr == p) {
            *old_meta = sh->tab[i].meta;
            sh->tab[i].meta = meta;
            return 2;
        }
        i = (i + 1) & (sh->cap - 1);
    }
    sh->tab[i].ptr = p;
    sh->tab[i].meta = meta;
    sh->count++;
    return 1;
}

/* Shard lock held. Backward-shift deletion keeps probe chains intact. */
static int tab_remove(shard_t *sh, uintptr_t p, uint64_t *meta)
{
    if (sh->tab == NULL)
        return 0;
    size_t cap = sh->cap;
    ent_t *t = sh->tab;
    size_t i = slot_of(p, cap);
    while (t[i].ptr != p) {
        if (t[i].ptr == 0)
            return 0;
        i = (i + 1) & (cap - 1);
    }
    *meta = t[i].meta;
    size_t j = i;
    for (;;) {
        j = (j + 1) & (cap - 1);
        if (t[j].ptr == 0)
            break;
        size_t home = slot_of(t[j].ptr, cap);
        /* t[j] may move back to i unless its home lies cyclically in (i, j] */
        int between = i <= j ? (home > i && home <= j) : (home > i || home <= j);
        if (!between) {
            t[i] = t[j];
            i = j;
        }
    }
    t[i].ptr = 0;
    t[i].meta = 0;
    sh->count--;
    return 1;
}

/* ---------------------------------------------------------------------------------------- */
/* stack (site) table: lock-free lookup, locked insert, atomic counters                       */

typedef struct {
    uintptr_t frames[KEEP_FRAMES]; /* immutable once published through site_idx */
    _Atomic int64_t live_bytes, live_blocks, allocs;
} site_t;

static site_t sites[MAX_SITES];
static _Atomic uint16_t site_idx[SITE_IDX]; /* slot + 1, 0 = empty */
static _Atomic unsigned sites_used;
static spin_t site_lock;
static _Atomic int64_t over_bytes, over_blocks, over_allocs;      /* SITE_NONE */
static _Atomic int64_t unsamp_bytes, unsamp_blocks, unsamp_allocs; /* SITE_UNSAMPLED */

static uint64_t hash_frames(const uintptr_t *f)
{
    uint64_t h = UINT64_C(0xcbf29ce484222325);
    for (int i = 0; i < KEEP_FRAMES; i++) {
        h ^= (uint64_t)f[i];
        h *= UINT64_C(0x100000001b3);
        h ^= h >> 29;
    }
    return h;
}

/* Probes for the stack; returns its id, or SITE_NONE; *empty gets the first free index slot. */
static unsigned site_probe(const uintptr_t *f, size_t *empty)
{
    size_t i = (size_t)(hash_frames(f) & (SITE_IDX - 1));
    for (;;) {
        unsigned v = atomic_load_explicit(&site_idx[i], memory_order_acquire);
        if (v == 0) {
            *empty = i;
            return SITE_NONE;
        }
        if (memcmp(sites[v - 1].frames, f, sizeof sites[v - 1].frames) == 0)
            return v - 1u;
        i = (i + 1) & (SITE_IDX - 1);
    }
}

static unsigned site_find_or_add(const uintptr_t *f)
{
    size_t empty = 0;
    unsigned id = site_probe(f, &empty);
    if (id != SITE_NONE)
        return id;
    spin_lock(&site_lock);
    id = site_probe(f, &empty); /* another thread may have added it */
    if (id == SITE_NONE) {
        unsigned n = atomic_load_explicit(&sites_used, memory_order_relaxed);
        if (n < MAX_SITES) {
            memcpy(sites[n].frames, f, sizeof sites[n].frames);
            atomic_store_explicit(&sites_used, n + 1, memory_order_relaxed);
            atomic_store_explicit(&site_idx[empty], (uint16_t)(n + 1), memory_order_release);
            id = n;
        }
    }
    spin_unlock(&site_lock);
    return id;
}

static void site_account(unsigned id, int64_t bytes, int64_t blocks, int64_t allocs)
{
    _Atomic int64_t *b, *k, *a;
    if (id == SITE_NONE) {
        b = &over_bytes, k = &over_blocks, a = &over_allocs;
    } else if (id == SITE_UNSAMPLED) {
        b = &unsamp_bytes, k = &unsamp_blocks, a = &unsamp_allocs;
    } else {
        b = &sites[id].live_bytes, k = &sites[id].live_blocks, a = &sites[id].allocs;
    }
    atomic_fetch_add_explicit(b, bytes, memory_order_relaxed);
    atomic_fetch_add_explicit(k, blocks, memory_order_relaxed);
    atomic_fetch_add_explicit(a, allocs, memory_order_relaxed);
}

/* ---------------------------------------------------------------------------------------- */
/* tracking                                                                                  */

static void bump_peak(int64_t now)
{
    int64_t p = atomic_load_explicit(&peak_bytes, memory_order_relaxed);
    while (now > p && !atomic_compare_exchange_weak_explicit(&peak_bytes, &p, now, memory_order_relaxed,
                                                             memory_order_relaxed))
        ;
}

/* Records a live block. Never allocates through malloc: backtrace() may, and the in_shim guard
 * (set by every caller) routes such nested calls to the real allocator untracked. */
__attribute__((noinline)) static void record_site(void *p, size_t size)
{
    if (p == NULL || atomic_load_explicit(&tab_broken, memory_order_relaxed))
        return;
    uintptr_t frames[KEEP_FRAMES] = {0};
    unsigned id = SITE_UNSAMPLED;
    if (sample_every <= 1 || (++sample_tick % (unsigned long)sample_every) == 0) {
        void *bt[STACK_DEPTH];
        int n = backtrace(bt, STACK_DEPTH);
        for (int i = SKIP_FRAMES; i < n && i - SKIP_FRAMES < KEEP_FRAMES; i++)
            frames[i - SKIP_FRAMES] = (uintptr_t)bt[i];
        id = site_find_or_add(frames);
    }
    uint64_t meta = ((uint64_t)id << 48) | ((uint64_t)size & SIZE_MASK);
    uint64_t old = 0;
    shard_t *sh = shard_of((uintptr_t)p);
    spin_lock(&sh->lock);
    int r = tab_insert(sh, (uintptr_t)p, meta, &old);
    spin_unlock(&sh->lock);
    if (r == 0) {
        atomic_fetch_add_explicit(&untracked_allocs, 1, memory_order_relaxed);
        return;
    }
    if (r == 2) { /* the earlier owner of this address vanished without our seeing it */
        site_account((unsigned)(old >> 48), -(int64_t)(old & SIZE_MASK), -1, 0);
        atomic_fetch_sub_explicit(&live_bytes, (int64_t)(old & SIZE_MASK), memory_order_relaxed);
        atomic_fetch_sub_explicit(&live_blocks, 1, memory_order_relaxed);
        atomic_fetch_add_explicit(&stale_replaced, 1, memory_order_relaxed);
    }
    site_account(id, (int64_t)size, 1, 1);
    int64_t now = atomic_fetch_add_explicit(&live_bytes, (int64_t)size, memory_order_relaxed) + (int64_t)size;
    atomic_fetch_add_explicit(&live_blocks, 1, memory_order_relaxed);
    atomic_fetch_add_explicit(&total_allocs, 1, memory_order_relaxed);
    bump_peak(now);
}

/* Removes a block from the books; returns whether it was known. */
static int track_free(void *p, uint64_t *meta_out)
{
    uint64_t meta = 0;
    shard_t *sh = shard_of((uintptr_t)p);
    spin_lock(&sh->lock);
    int found = tab_remove(sh, (uintptr_t)p, &meta);
    spin_unlock(&sh->lock);
    if (found) {
        int64_t sz = (int64_t)(meta & SIZE_MASK);
        site_account((unsigned)(meta >> 48), -sz, -1, 0);
        atomic_fetch_sub_explicit(&live_bytes, sz, memory_order_relaxed);
        atomic_fetch_sub_explicit(&live_blocks, 1, memory_order_relaxed);
    } else {
        atomic_fetch_add_explicit(&unknown_frees, 1, memory_order_relaxed);
    }
    if (meta_out)
        *meta_out = meta;
    return found;
}

/* ---------------------------------------------------------------------------------------- */
/* resolving the real allocator                                                              */

static void resolve_real(void)
{
    int expected = ST_UNINIT;
    if (!atomic_compare_exchange_strong(&resolve_state, &expected, ST_RESOLVING))
        return; /* someone else is resolving; callers use the arena until READY */
#ifdef __APPLE__
    real_malloc = malloc;
    real_free = free;
    real_calloc = calloc;
    real_realloc = realloc;
    real_posix_memalign = posix_memalign;
#else
    real_free = (void (*)(void *))dlsym(RTLD_NEXT, "free");
    real_calloc = (void *(*)(size_t, size_t))dlsym(RTLD_NEXT, "calloc");
    real_realloc = (void *(*)(void *, size_t))dlsym(RTLD_NEXT, "realloc");
    real_posix_memalign = (int (*)(void **, size_t, size_t))dlsym(RTLD_NEXT, "posix_memalign");
    real_malloc = (void *(*)(size_t))dlsym(RTLD_NEXT, "malloc");
#endif
    atomic_store(&resolve_state, ST_READY);
}

static inline int ready(void)
{
    if (__builtin_expect(atomic_load_explicit(&resolve_state, memory_order_acquire) == ST_READY, 1))
        return 1;
    resolve_real();
    return atomic_load_explicit(&resolve_state, memory_order_acquire) == ST_READY;
}

/* ---------------------------------------------------------------------------------------- */
/* interposed entry points                                                                   */

#ifdef __APPLE__
#define ENTRY(name) dla_##name
#else
#define ENTRY(name) name
#endif

DLA_EXPORT void *ENTRY(malloc)(size_t n)
{
    if (!ready())
        return arena_alloc(n);
    if (in_shim || !active)
        return real_malloc(n);
    in_shim = 1;
    void *p = real_malloc(n);
    record_site(p, n);
    in_shim = 0;
    return p;
}

DLA_EXPORT void ENTRY(free)(void *p)
{
    if (p == NULL)
        return;
    if (in_arena(p))
        return;
    if (!ready())
        return;
    if (active)
        track_free(p, NULL); /* before the real free: see the file comment */
    real_free(p);
}

DLA_EXPORT void *ENTRY(calloc)(size_t a, size_t b)
{
    if (!ready()) {
        size_t n;
        if (__builtin_mul_overflow(a, b, &n)) {
            errno = ENOMEM;
            return NULL;
        }
        return arena_alloc(n);
    }
    if (in_shim || !active)
        return real_calloc(a, b);
    in_shim = 1;
    void *p = real_calloc(a, b);
    record_site(p, a * b); /* a * b cannot overflow when calloc succeeded */
    in_shim = 0;
    return p;
}

DLA_EXPORT void *ENTRY(realloc)(void *old, size_t n)
{
    if (old == NULL)
        return ENTRY(malloc)(n);
    if (in_arena(old)) {
        size_t osz = *(size_t *)((unsigned char *)old - 16);
        void *p = ENTRY(malloc)(n);
        if (p != NULL)
            memcpy(p, old, osz < n ? osz : n);
        return p;
    }
    if (!ready())
        return NULL;
    if (in_shim || !active)
        return real_realloc(old, n);
    in_shim = 1;
    /* Unregister first: once real_realloc moves or frees the block another thread may be
     * handed the old address and register it before we would get to remove our entry. */
    uint64_t meta = 0;
    int known = track_free(old, &meta);
    void *p = real_realloc(old, n);
    if (p != NULL) {
        record_site(p, n);
    } else if (n != 0 && known) { /* failed: the old block is still live and still ours */
        void *keep = old;
        size_t osz = (size_t)(meta & SIZE_MASK);
        record_site(keep, osz); /* site stack is the realloc caller now; sizes stay exact */
    }
    in_shim = 0;
    return p;
}

static int aligned_common(void **out, size_t align, size_t n)
{
    /* aligned_alloc and memalign accept alignments below sizeof(void *) (C++ new with a
     * small align_val_t does this), posix_memalign does not: a larger power of two is
     * as good. */
    if (align != 0 && (align & (align - 1)) == 0 && align < sizeof(void *))
        align = sizeof(void *);
    if (!ready()) {
        *out = arena_alloc(n + align);
        if (*out == NULL)
            return ENOMEM;
        uintptr_t a = ((uintptr_t)*out + align - 1) & ~(uintptr_t)(align - 1);
        *out = (void *)a;
        return 0;
    }
    if (in_shim || !active)
        return real_posix_memalign(out, align, n);
    in_shim = 1;
    void *p = NULL;
    int rc = real_posix_memalign(&p, align, n);
    if (rc == 0)
        record_site(p, n);
    *out = p;
    in_shim = 0;
    return rc;
}

DLA_EXPORT int ENTRY(posix_memalign)(void **out, size_t align, size_t n)
{
    return aligned_common(out, align, n);
}

static void *aligned_errno(size_t align, size_t n)
{
    void *p = NULL;
    int rc = aligned_common(&p, align, n);
    if (rc != 0) {
        errno = rc;
        return NULL;
    }
    return p;
}

DLA_EXPORT void *ENTRY(aligned_alloc)(size_t align, size_t n) { return aligned_errno(align, n); }

#ifndef __APPLE__
DLA_EXPORT void *memalign(size_t align, size_t n) { return aligned_errno(align, n); }

DLA_EXPORT void *valloc(size_t n) { return aligned_errno((size_t)sysconf(_SC_PAGESIZE), n); }

DLA_EXPORT void *pvalloc(size_t n)
{
    size_t pg = (size_t)sysconf(_SC_PAGESIZE);
    if (n > SIZE_MAX - pg) {
        errno = ENOMEM;
        return NULL;
    }
    return aligned_errno(pg, (n + pg - 1) & ~(pg - 1));
}

DLA_EXPORT void *reallocarray(void *p, size_t a, size_t b)
{
    size_t n;
    if (__builtin_mul_overflow(a, b, &n)) {
        errno = ENOMEM;
        return NULL;
    }
    return realloc(p, n);
}
#endif

#ifdef __APPLE__
#define INTERPOSE(rep, orig) \
    __attribute__((used)) static struct { const void *r; const void *o; } interpose_##orig \
        __attribute__((section("__DATA,__interpose"))) = {(const void *)(rep), (const void *)(orig)}
INTERPOSE(dla_malloc, malloc);
INTERPOSE(dla_free, free);
INTERPOSE(dla_calloc, calloc);
INTERPOSE(dla_realloc, realloc);
INTERPOSE(dla_posix_memalign, posix_memalign);
INTERPOSE(dla_aligned_alloc, aligned_alloc);
#endif

/* ---------------------------------------------------------------------------------------- */
/* JSON output without malloc or stdio                                                       */

static char obuf[1 << 20];
static size_t olen;
static int ofd = -1;

static void oflush(void)
{
    size_t off = 0;
    while (off < olen) {
        ssize_t w = write(ofd, obuf + off, olen - off);
        if (w < 0) {
            if (errno == EINTR)
                continue;
            break;
        }
        off += (size_t)w;
    }
    olen = 0;
}

static void putn(const char *s, size_t n)
{
    while (n > 0) {
        size_t room = sizeof obuf - olen;
        if (room == 0) {
            oflush();
            room = sizeof obuf;
        }
        size_t c = n < room ? n : room;
        memcpy(obuf + olen, s, c);
        olen += c;
        s += c;
        n -= c;
    }
}

#define PUTL(s) putn((s), sizeof(s) - 1)

static void putu(uint64_t v)
{
    char t[24];
    int i = 24;
    do {
        t[--i] = (char)('0' + v % 10);
        v /= 10;
    } while (v);
    putn(t + i, (size_t)(24 - i));
}

static void puti(int64_t v)
{
    if (v < 0) {
        PUTL("-");
        putu((uint64_t)0 - (uint64_t)v);
    } else {
        putu((uint64_t)v);
    }
}

static void puthex(uint64_t v) /* "0x..." without quotes */
{
    char t[20];
    int i = 20;
    do {
        t[--i] = "0123456789abcdef"[v & 15];
        v >>= 4;
    } while (v);
    PUTL("0x");
    putn(t + i, (size_t)(20 - i));
}

static void putjs(const char *s) /* quoted JSON string */
{
    PUTL("\"");
    for (; *s; s++) {
        unsigned char c = (unsigned char)*s;
        if (c == '"' || c == '\\') {
            char e[2] = {'\\', (char)c};
            putn(e, 2);
        } else if (c < 0x20) {
            char e[7] = {'\\', 'u', '0', '0', "0123456789abcdef"[c >> 4], "0123456789abcdef"[c & 15], 0};
            putn(e, 6);
        } else {
            putn((const char *)&c, 1);
        }
    }
    PUTL("\"");
}

static void put_kv_i(const char *key, int64_t v, int *first)
{
    if (v < 0)
        return; /* unreadable value: leave the key out */
    if (!*first)
        PUTL(",");
    *first = 0;
    putjs(key);
    PUTL(":");
    puti(v);
}

/* ---------------------------------------------------------------------------------------- */
/* file descriptor kinds                                                                     */

/* The kinds follow the sampler's fd.<kind> columns (crates/dla-sample/src/os/fd.rs):
 * socket, pipe, file, deleted, dev, eventfd, eventpoll, timerfd (always reported), plus any
 * other kind that has a descriptor (memfd, inotify, ..., other). */
#define FD_KIND_LEN 32
#define FD_KINDS 24

typedef struct {
    char name[FD_KIND_LEN];
    int64_t n;
} fdk_t;

static fdk_t fd_kinds[FD_KINDS];
static int n_fd_kinds;
static int fd_kinds_valid;
static const char *const fd_base_kinds[] = {"socket", "pipe", "file", "deleted", "dev", "eventfd", "eventpoll", "timerfd"};

static void fd_kinds_reset(void)
{
    n_fd_kinds = 0;
    for (size_t i = 0; i < sizeof fd_base_kinds / sizeof fd_base_kinds[0]; i++) {
        snprintf(fd_kinds[n_fd_kinds].name, FD_KIND_LEN, "%s", fd_base_kinds[i]);
        fd_kinds[n_fd_kinds++].n = 0;
    }
}

static void fd_kind_add(const char *kind)
{
    for (int i = 0; i < n_fd_kinds; i++) {
        if (strcmp(fd_kinds[i].name, kind) == 0) {
            fd_kinds[i].n++;
            return;
        }
    }
    if (n_fd_kinds < FD_KINDS) {
        snprintf(fd_kinds[n_fd_kinds].name, FD_KIND_LEN, "%s", kind);
        fd_kinds[n_fd_kinds++].n = 1;
    } else {
        fd_kinds[FD_KINDS - 1].n++; /* the last slot collects the overflow */
    }
}

static int has_suffix(const char *s, const char *suffix)
{
    size_t a = strlen(s), b = strlen(suffix);
    return a >= b && strcmp(s + a - b, suffix) == 0;
}

/* Kind of one /proc/self/fd link target. */
static void fd_classify(const char *t, char *kind)
{
    while (*t == ' ')
        t++;
    if (strncmp(t, "anon_inode:", 11) == 0) {
        const char *in = t + 11;
        size_t k = 0;
        while (*in == '[')
            in++;
        for (; *in && k + 1 < FD_KIND_LEN; in++) {
            char c = *in;
            if (c == ']')
                continue;
            if (c >= 'A' && c <= 'Z')
                c = (char)(c - 'A' + 'a');
            kind[k++] = ((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '_') ? c : '_';
        }
        kind[k] = 0;
        size_t lo = 0;
        while (kind[lo] == '_')
            lo++;
        size_t hi = strlen(kind);
        while (hi > lo && kind[hi - 1] == '_')
            hi--;
        memmove(kind, kind + lo, hi - lo);
        kind[hi - lo] = 0;
        if (kind[0] == 0)
            snprintf(kind, FD_KIND_LEN, "anon_inode");
        return;
    }
    const char *k;
    if (strncmp(t, "socket:[", 8) == 0)
        k = "socket";
    else if (strncmp(t, "pipe:[", 6) == 0)
        k = "pipe";
    else if (strncmp(t, "/memfd:", 7) == 0)
        k = "memfd";
    else if (has_suffix(t, " (deleted)"))
        k = "deleted";
    else if (strncmp(t, "/dev/", 5) == 0)
        k = "dev";
    else if (t[0] == '/')
        k = "file";
    else
        k = "other";
    snprintf(kind, FD_KIND_LEN, "%s", k);
}

static void put_fd_kinds(int *first)
{
    if (!fd_kinds_valid)
        return;
    for (int i = 0; i < n_fd_kinds; i++) {
        char key[FD_KIND_LEN + 4];
        snprintf(key, sizeof key, "fd.%.31s", fd_kinds[i].name);
        if (i < (int)(sizeof fd_base_kinds / sizeof fd_base_kinds[0]) || fd_kinds[i].n > 0)
            put_kv_i(key, fd_kinds[i].n, first);
    }
}

/* ---------------------------------------------------------------------------------------- */
/* OS values                                                                                 */

typedef struct {
    int64_t rss, rss_anon, rss_file, rss_shmem, swap, vm_data, vm_hwm, threads, fds;
} osv_t;

#ifdef __linux__
static int64_t status_kb(const char *buf, const char *key)
{
    size_t kl = strlen(key);
    for (const char *p = buf; *p;) {
        if (strncmp(p, key, kl) == 0) {
            p += kl;
            while (*p == ' ' || *p == '\t')
                p++;
            int64_t v = 0;
            int any = 0;
            while (*p >= '0' && *p <= '9') {
                v = v * 10 + (*p++ - '0');
                any = 1;
            }
            return any ? v : -1;
        }
        while (*p && *p != '\n')
            p++;
        if (*p)
            p++;
    }
    return -1;
}
#endif

static int64_t kb_to_bytes(int64_t kb) { return kb < 0 ? -1 : kb * 1024; }

static void read_os(osv_t *o, int with_fds)
{
    o->rss = o->rss_anon = o->rss_file = o->rss_shmem = o->swap = o->vm_data = o->vm_hwm = o->threads =
        o->fds = -1;
#ifdef __linux__
    static char sbuf[8192];
    int fd = open("/proc/self/status", O_RDONLY | O_CLOEXEC);
    if (fd >= 0) {
        ssize_t n = read(fd, sbuf, sizeof sbuf - 1);
        close(fd);
        if (n > 0) {
            sbuf[n] = 0;
            o->rss = kb_to_bytes(status_kb(sbuf, "VmRSS:"));
            o->rss_anon = kb_to_bytes(status_kb(sbuf, "RssAnon:"));
            o->rss_file = kb_to_bytes(status_kb(sbuf, "RssFile:"));
            o->rss_shmem = kb_to_bytes(status_kb(sbuf, "RssShmem:"));
            o->swap = kb_to_bytes(status_kb(sbuf, "VmSwap:"));
            o->vm_data = kb_to_bytes(status_kb(sbuf, "VmData:"));
            o->vm_hwm = kb_to_bytes(status_kb(sbuf, "VmHWM:"));
            o->threads = status_kb(sbuf, "Threads:");
        }
    }
    if (with_fds) {
        int saved = in_shim;
        in_shim = 1; /* opendir allocates; keep it out of the books */
        DIR *d = opendir("/proc/self/fd");
        if (d != NULL) {
            int self = dirfd(d);
            int64_t count = 0;
            fd_kinds_reset();
            struct dirent *de;
            while ((de = readdir(d)) != NULL) {
                if (de->d_name[0] == '.')
                    continue;
                if (atoi(de->d_name) == self)
                    continue; /* the directory's own descriptor */
                char target[512];
                ssize_t n = readlinkat(self, de->d_name, target, sizeof target - 1);
                if (n < 0)
                    continue; /* closed while the directory was read */
                target[n] = 0;
                count++;
                char kind[FD_KIND_LEN];
                fd_classify(target, kind);
                fd_kind_add(kind);
            }
            closedir(d);
            o->fds = count;
            fd_kinds_valid = 1;
        }
        in_shim = saved;
    }
#else
    (void)with_fds;
#endif
}

static void put_os_members(const osv_t *o, int *first)
{
    put_kv_i("os.rss", o->rss, first);
    put_kv_i("os.rss_anon", o->rss_anon, first);
    put_kv_i("os.rss_file", o->rss_file, first);
    put_kv_i("os.rss_shmem", o->rss_shmem, first);
    put_kv_i("os.swap", o->swap, first);
    put_kv_i("os.vm_data", o->vm_data, first);
    put_kv_i("os.vm_hwm", o->vm_hwm, first);
    put_kv_i("os.threads", o->threads, first);
    put_kv_i("os.fds", o->fds, first);
}

static void put_os(const osv_t *o)
{
    int first = 1;
    PUTL("{");
    put_os_members(o, &first);
    PUTL("}");
}

/* ---------------------------------------------------------------------------------------- */
/* modules                                                                                   */

typedef struct {
    char path[512];
    uintptr_t bias; /* subtract from a runtime address to get the address addr2line expects */
    uintptr_t lo[4], hi[4];
    int nseg;
} module_t;

static module_t modules[MAX_MODULES];
static int nmodules;
static char exe_path[512];

#ifdef __linux__
static int phdr_cb(struct dl_phdr_info *info, size_t size, void *data)
{
    (void)size;
    (void)data;
    if (nmodules >= MAX_MODULES)
        return 0;
    module_t *m = &modules[nmodules];
    const char *name = info->dlpi_name;
    if (name == NULL || name[0] == 0)
        name = exe_path[0] ? exe_path : "";
    snprintf(m->path, sizeof m->path, "%s", name);
    m->bias = (uintptr_t)info->dlpi_addr;
    m->nseg = 0;
    for (int i = 0; i < info->dlpi_phnum && m->nseg < 4; i++) {
        const ElfW(Phdr) *ph = &info->dlpi_phdr[i];
        if (ph->p_type != PT_LOAD)
            continue;
        m->lo[m->nseg] = (uintptr_t)info->dlpi_addr + ph->p_vaddr;
        m->hi[m->nseg] = m->lo[m->nseg] + ph->p_memsz;
        m->nseg++;
    }
    if (m->nseg > 0)
        nmodules++;
    return 0;
}

static void collect_modules(void)
{
    nmodules = 0;
    dl_iterate_phdr(phdr_cb, NULL);
}

static int module_of(uintptr_t a)
{
    for (int i = 0; i < nmodules; i++)
        for (int s = 0; s < modules[i].nseg; s++)
            if (a >= modules[i].lo[s] && a < modules[i].hi[s])
                return i;
    return -1;
}
#else
static void collect_modules(void) { nmodules = 0; }

static int module_of(uintptr_t a)
{
    Dl_info di;
    if (!dladdr((void *)a, &di) || di.dli_fname == NULL)
        return -1;
    for (int i = 0; i < nmodules; i++)
        if (modules[i].bias == (uintptr_t)di.dli_fbase)
            return i;
    if (nmodules >= MAX_MODULES)
        return -1;
    module_t *m = &modules[nmodules];
    strncpy(m->path, di.dli_fname, sizeof m->path - 1);
    m->path[sizeof m->path - 1] = 0;
    m->bias = (uintptr_t)di.dli_fbase;
    m->nseg = 0;
    return nmodules++;
}
#endif

/* ---------------------------------------------------------------------------------------- */
/* dump                                                                                      */

static _Atomic int dumped;
static const char *out_path;
static const char *purge_kind;
static const char *iter_env;
typedef struct {
    uintptr_t frames[KEEP_FRAMES];
    int64_t live_bytes, live_blocks, allocs;
} snap_t;

static snap_t snap[MAX_SITES];
static uint16_t order[MAX_SITES];

static int cmp_site(const void *a, const void *b)
{
    const snap_t *x = &snap[*(const uint16_t *)a], *y = &snap[*(const uint16_t *)b];
    if (x->live_bytes != y->live_bytes)
        return x->live_bytes < y->live_bytes ? 1 : -1;
    return 0;
}

static void put_cmdline(void)
{
    static char cl[32768];
    PUTL("[");
#ifdef __linux__
    int fd = open("/proc/self/cmdline", O_RDONLY | O_CLOEXEC);
    if (fd >= 0) {
        ssize_t n = read(fd, cl, sizeof cl - 1);
        close(fd);
        int first = 1;
        for (ssize_t i = 0; i < n;) {
            if (!first)
                PUTL(",");
            first = 0;
            putjs(cl + i);
            i += (ssize_t)strlen(cl + i) + 1;
        }
    }
#endif
    PUTL("]");
}

static void put_frame(uintptr_t a)
{
    PUTL("{\"a\":");
    int m = a ? module_of(a) : -1;
    PUTL("\"");
    puthex(a);
    PUTL("\"");
    if (m >= 0) {
        PUTL(",\"m\":");
        putu((uint64_t)m);
        PUTL(",\"o\":\"");
        puthex((a - 1) - modules[m].bias);
        PUTL("\"");
    }
    PUTL("}");
}

static void dump_record(int from_signal)
{
    if (atomic_exchange(&dumped, 1))
        return;
    if (!active || out_path == NULL || getpid() != init_pid)
        return;
    int64_t lb = atomic_load(&live_bytes), lk = atomic_load(&live_blocks), pk = atomic_load(&peak_bytes);
    unsigned nused = atomic_load(&sites_used);
    for (unsigned i = 0; i < nused; i++) { /* counters are atomics: no lock is needed (or taken) */
        memcpy(snap[i].frames, sites[i].frames, sizeof snap[i].frames);
        snap[i].live_bytes = atomic_load(&sites[i].live_bytes);
        snap[i].live_blocks = atomic_load(&sites[i].live_blocks);
        snap[i].allocs = atomic_load(&sites[i].allocs);
    }
    int64_t ob = atomic_load(&over_bytes), okb = atomic_load(&over_blocks), oa = atomic_load(&over_allocs);
    int64_t ub = atomic_load(&unsamp_bytes), ukb = atomic_load(&unsamp_blocks), ua = atomic_load(&unsamp_allocs);

    int saved = in_shim;
    in_shim = 1;
    osv_t os, os_after;
    read_os(&os, !from_signal);
    const char *limit[8] = {0};
    int nlim = 0;
    int have_purge = 0;
    const char *purge_detail = NULL;
    if (purge_kind != NULL && strcmp(purge_kind, "none") != 0 && purge_kind[0] != 0) {
        if (strcmp(purge_kind, "trim") == 0) {
#ifdef __GLIBC__
            malloc_trim(0);
            have_purge = 1;
            purge_detail = "malloc_trim(0)";
#elif defined(__APPLE__)
            malloc_zone_pressure_relief(NULL, 0);
            have_purge = 1;
            purge_detail = "malloc_zone_pressure_relief";
#else
            limit[nlim++] = "purge trim is not available: libc has no malloc_trim";
#endif
        } else if (strcmp(purge_kind, "jemalloc") == 0) {
            int (*mc)(const char *, void *, size_t *, void *, size_t) =
                (int (*)(const char *, void *, size_t *, void *, size_t))dlsym(RTLD_DEFAULT, "mallctl");
            if (mc == NULL)
                mc = (int (*)(const char *, void *, size_t *, void *, size_t))dlsym(RTLD_DEFAULT, "je_mallctl");
            if (mc != NULL && mc("arena.4096.purge", NULL, NULL, NULL, 0) == 0) {
                have_purge = 1;
                purge_detail = "mallctl arena.4096.purge";
            } else {
                limit[nlim++] = "purge jemalloc requested but the process has no jemalloc mallctl";
            }
        } else {
            limit[nlim++] = "purge kind is not supported by the allocator shim (supported: trim, jemalloc)";
        }
        if (have_purge)
            read_os(&os_after, 0);
    }

    collect_modules();
    unsigned cnt = 0;
    for (unsigned i = 0; i < nused; i++)
        if (snap[i].live_blocks > 0)
            order[cnt++] = (uint16_t)i;
    qsort(order, cnt, sizeof order[0], cmp_site);
    unsigned shown = cnt < MAX_DUMP_SITES ? cnt : MAX_DUMP_SITES;

    struct timespec t1;
    clock_gettime(CLOCK_MONOTONIC, &t1);
    int64_t t_ms = (int64_t)(t1.tv_sec - t0.tv_sec) * 1000 + (t1.tv_nsec - t0.tv_nsec) / 1000000;

    ofd = open(out_path, O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC, 0644);
    if (ofd < 0) {
        in_shim = saved;
        return;
    }
    olen = 0;
    PUTL("{\"dla_harness\":1,\"phase\":\"exit\",\"pid\":");
    putu((uint64_t)getpid());
    PUTL(",\"argv\":");
    put_cmdline();
    PUTL(",\"exe\":");
    putjs(exe_path);
    PUTL(",\"runtime\":\"native\"");
    if (iter_env != NULL && iter_env[0] >= '0' && iter_env[0] <= '9') {
        PUTL(",\"x\":");
        putu((uint64_t)strtoull(iter_env, NULL, 10));
    }
    PUTL(",\"t_ms\":");
    puti(t_ms);
    PUTL(",\"gc_forced\":false,\"values\":{");
    int first = 1;
    put_kv_i("native.live_bytes", lb, &first);
    put_kv_i("native.live_blocks", lk, &first);
    put_kv_i("native.peak_bytes", pk, &first);
    put_os_members(&os, &first);
    put_fd_kinds(&first);
    PUTL("}");
    if (have_purge) {
        PUTL(",\"purge\":{\"kind\":");
        putjs(purge_kind);
        PUTL(",\"before\":");
        put_os(&os);
        PUTL(",\"after\":");
        put_os(&os_after);
        PUTL(",\"detail\":");
        putjs(purge_detail);
        PUTL("}");
    }
    PUTL(",\"groups\":{},\"raw_sites\":{");
    for (unsigned k = 0; k < shown; k++) {
        const snap_t *s = &snap[order[k]];
        if (k)
            PUTL(",");
        PUTL("\"");
        puthex(hash_frames(s->frames));
        PUTL("\":{\"live_bytes\":");
        puti(s->live_bytes);
        PUTL(",\"live_blocks\":");
        puti(s->live_blocks);
        PUTL(",\"allocs\":");
        puti(s->allocs);
        PUTL(",\"frames\":[");
        int nf = 0;
        while (nf < KEEP_FRAMES && s->frames[nf] != 0)
            nf++;
        for (int f = 0; f < nf; f++) {
            if (f)
                PUTL(",");
            put_frame(s->frames[f]);
        }
        PUTL("]}");
    }
    PUTL("},\"modules\":[");
    for (int i = 0; i < nmodules; i++) {
        if (i)
            PUTL(",");
        PUTL("{\"path\":");
        putjs(modules[i].path);
        PUTL(",\"bias\":\"");
        puthex(modules[i].bias);
        PUTL("\"}");
    }
    PUTL("],\"shim\":{\"sample\":");
    putu((uint64_t)sample_every);
    PUTL(",\"sites_used\":");
    putu(nused);
    PUTL(",\"sites_live\":");
    putu(cnt);
    PUTL(",\"sites_shown\":");
    putu(shown);
    PUTL(",\"overflow_live_bytes\":");
    puti(ob);
    PUTL(",\"overflow_live_blocks\":");
    puti(okb);
    PUTL(",\"overflow_allocs\":");
    puti(oa);
    PUTL(",\"unsampled_live_bytes\":");
    puti(ub);
    PUTL(",\"unsampled_live_blocks\":");
    puti(ukb);
    PUTL(",\"unsampled_allocs\":");
    puti(ua);
    PUTL(",\"total_allocs\":");
    puti(atomic_load(&total_allocs));
    PUTL(",\"untracked_allocs\":");
    puti(atomic_load(&untracked_allocs));
    PUTL(",\"stale_replaced\":");
    puti(atomic_load(&stale_replaced));
    PUTL(",\"unknown_frees\":");
    puti(atomic_load(&unknown_frees));
    PUTL("},\"limitations\":[");
    int firstl = 1;
    if (cnt > shown) {
        if (!firstl)
            PUTL(",");
        firstl = 0;
        putjs("raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted");
    }
    if (ob != 0 || okb != 0 || oa != 0) {
        if (!firstl)
            PUTL(",");
        firstl = 0;
        putjs("more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*");
    }
    if (sample_every > 1) {
        if (!firstl)
            PUTL(",");
        firstl = 0;
        putjs("DLA_SHIM_SAMPLE > 1: raw_sites cover sampled allocations only; native.* totals are exact");
    }
    if (atomic_load(&untracked_allocs) != 0) {
        if (!firstl)
            PUTL(",");
        firstl = 0;
        putjs("some allocations were not tracked (pointer table could not grow); totals are low");
    }
    if (atomic_load(&tab_broken)) {
        if (!firstl)
            PUTL(",");
        firstl = 0;
        putjs("pointer table could not be mapped; native.* totals are not reliable");
    }
    for (int i = 0; i < nlim; i++) {
        if (!firstl)
            PUTL(",");
        firstl = 0;
        putjs(limit[i]);
    }
    PUTL("]}\n");
    oflush();
    close(ofd);
    ofd = -1;
    in_shim = saved;
}

/* ---------------------------------------------------------------------------------------- */
/* init                                                                                      */

static void on_sigterm(int sig)
{
    signal(SIGALRM, SIG_DFL);
    alarm(3); /* if the dump deadlocks inside the loader, the default SIGALRM action ends us */
    dump_record(1);
    signal(sig, SIG_DFL);
    raise(sig);
}

static void fork_lock_all(void)
{
    spin_lock(&site_lock);
    for (int i = 0; i < SHARDS; i++)
        spin_lock(&shards[i].lock);
}

static void fork_unlock_all(void)
{
    for (int i = SHARDS - 1; i >= 0; i--)
        spin_unlock(&shards[i].lock);
    spin_unlock(&site_lock);
}

static const char *base_name(const char *p)
{
    const char *s = strrchr(p, '/');
    return s ? s + 1 : p;
}

__attribute__((constructor(101))) static void dla_shim_init(void)
{
    ready(); /* resolve the real allocator while the process is still single-threaded */
    clock_gettime(CLOCK_MONOTONIC, &t0);
    init_pid = getpid();
    out_path = getenv("DLA_HARNESS_OUT");
    purge_kind = getenv("DLA_PURGE");
    iter_env = getenv("DLA_ITER");
    if (out_path == NULL || out_path[0] == 0)
        return;
    const char *off = getenv("DLA_SHIM_DISABLE");
    if (off != NULL && off[0] == '1')
        return; /* a driver in this process writes its own records */
#ifdef __linux__
    ssize_t n = readlink("/proc/self/exe", exe_path, sizeof exe_path - 1);
    exe_path[n > 0 ? n : 0] = 0;
#endif
    const char *only = getenv("DLA_SHIM_ONLY");
    if (only != NULL && only[0] != 0) {
        const char *eb = base_name(exe_path);
        if (strcmp(eb, only) != 0 && strcmp(base_name(only), eb) != 0)
            return;
    }
    const char *sm = getenv("DLA_SHIM_SAMPLE");
    if (sm != NULL && sm[0] != 0) {
        long k = strtol(sm, NULL, 10);
        sample_every = k > 1 ? (k > 1000000 ? 1000000 : (int)k) : 1;
    }
    const char *sg = getenv("DLA_SHIM_SIGTERM");
    if (sg != NULL && sg[0] == '0')
        sigterm_enabled = 0;
    {
        void *warm[2]; /* first backtrace() loads libgcc and may allocate: do it before tracking */
        in_shim = 1;
        backtrace(warm, 2);
        in_shim = 0;
    }
    pthread_atfork(fork_lock_all, fork_unlock_all, fork_unlock_all);
    if (sigterm_enabled) {
        struct sigaction cur;
        if (sigaction(SIGTERM, NULL, &cur) == 0 && cur.sa_handler == SIG_DFL && !(cur.sa_flags & SA_SIGINFO)) {
            struct sigaction sa;
            memset(&sa, 0, sizeof sa);
            sa.sa_handler = on_sigterm;
            sigemptyset(&sa.sa_mask);
            sigaction(SIGTERM, &sa, NULL);
        }
    }
    active = 1;
}

__attribute__((destructor(101))) static void dla_shim_fini(void) { dump_record(0); }
