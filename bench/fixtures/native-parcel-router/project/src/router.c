#include <stdio.h>
#include <string.h>

#include "types.h"

#define LABEL_BUCKETS 4096u

int router_init(router *r, const char *spool_path)
{
    memset(r, 0, sizeof *r);
    r->spool_path = spool_path;
    return label_pool_init(&r->labels, LABEL_BUCKETS);
}

static int hub_index(const char *hub)
{
    for (unsigned i = 0; i < HUB_COUNT; i++) {
        if (strcmp(hub, hub_name(i)) == 0)
            return (int)i;
    }
    return -1;
}

/* Stages the manifest in the spool, checks it and counts the parcel for its
 * destination hub. */
route_status router_handle(router *r, const parcel *p)
{
    const char *tag = label_intern(&r->labels, p->hub, p->lane, p->seq);
    if (tag == NULL)
        return ROUTE_ERR_SPOOL;

    spool *sp = spool_open(r->spool_path);
    if (sp == NULL)
        return ROUTE_ERR_SPOOL;

    int hub = hub_index(p->hub);
    if (hub < 0) {
        spool_close(sp);
        r->rejected++;
        return ROUTE_ERR_HUB;
    }

    if (manifest_sum(p->manifest, p->manifest_len) != p->sum) {
        r->rejected++;
        return ROUTE_ERR_CHECKSUM;
    }

    if (spool_write(sp, p->manifest, p->manifest_len) != 0) {
        spool_close(sp);
        return ROUTE_ERR_SPOOL;
    }
    spool_close(sp);
    r->per_hub[hub]++;
    r->routed++;
    return ROUTE_OK;
}
