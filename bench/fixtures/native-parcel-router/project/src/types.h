#ifndef TYPES_H
#define TYPES_H

#include <stddef.h>
#include <stdint.h>

/* A sorting hub reads parcel records from its feed. Each record names a
 * destination hub, carries a manifest with a checksum, and is staged in a
 * spool file while the router validates it. */

#define HUB_COUNT 8
#define MANIFEST_MAX 240

typedef struct {
    uint32_t seq;
    char hub[8];            /* destination hub code */
    uint16_t lane;
    uint16_t manifest_len;
    uint32_t sum;           /* checksum of the manifest */
    uint8_t manifest[MANIFEST_MAX];
} parcel;

typedef enum {
    ROUTE_OK = 0,
    ROUTE_ERR_HUB,
    ROUTE_ERR_CHECKSUM,
    ROUTE_ERR_SPOOL
} route_status;

/* parcel.c: deterministic feed used by the workload and the tests */
typedef struct {
    uint64_t state;
    uint32_t next_seq;
    uint32_t damage_every;  /* every n-th record has a damaged manifest; 0 = none */
    uint32_t unknown_every; /* every n-th record names a hub that does not exist; 0 = none */
} feed;

void feed_init(feed *f, uint64_t seed, uint32_t damage_every, uint32_t unknown_every);
void feed_next(feed *f, parcel *out);
uint32_t manifest_sum(const uint8_t *data, size_t len);
const char *hub_name(unsigned index);

/* spool.c: a staging file with its own buffer */
typedef struct spool spool;

spool *spool_open(const char *path);
int spool_write(spool *s, const void *data, size_t len);
void spool_close(spool *s);

/* intern.c: short names for the trace log */
typedef struct label label;

typedef struct {
    label **buckets;
    size_t bucket_count;
    size_t entries;
} label_pool;

int label_pool_init(label_pool *p, size_t bucket_count);
const char *label_intern(label_pool *p, const char *hub, uint16_t lane, uint32_t seq);
size_t label_pool_entries(const label_pool *p);

/* router.c */
typedef struct {
    const char *spool_path;
    label_pool labels;
    unsigned long routed;
    unsigned long rejected;
    unsigned long per_hub[HUB_COUNT];
} router;

int router_init(router *r, const char *spool_path);
route_status router_handle(router *r, const parcel *p);

/* report.c */
void report_summary(const router *r, unsigned long iterations);

#endif
