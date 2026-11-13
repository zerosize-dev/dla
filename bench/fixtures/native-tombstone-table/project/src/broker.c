#include <stdlib.h>

#include "types.h"

#define INITIAL_SLOTS 1024u

uint64_t broker_rand(broker *b)
{
    b->rng ^= b->rng << 13;
    b->rng ^= b->rng >> 7;
    b->rng ^= b->rng << 17;
    return b->rng;
}

int broker_init(broker *b, size_t window, uint64_t seed)
{
    b->active = malloc((window + 1) * sizeof *b->active);
    if (b->active == NULL)
        return -1;
    if (lease_table_init(&b->table, INITIAL_SLOTS) != 0) {
        free(b->active);
        b->active = NULL;
        return -1;
    }
    b->active_len = 0;
    b->window = window;
    b->next_id = 1000;
    b->clock = 0;
    b->rng = seed ? seed : 88172645463325252ULL;
    b->granted = 0;
    b->released = 0;
    return 0;
}

void broker_destroy(broker *b)
{
    lease_table_destroy(&b->table);
    free(b->active);
    b->active = NULL;
    b->active_len = 0;
}

int broker_grant(broker *b, uint32_t owner, uint64_t ttl)
{
    lease_slot e;
    e.key = b->next_id++;
    e.owner = owner;
    e.state = SLOT_LIVE;
    e.granted_at = b->clock;
    e.expires_at = b->clock + ttl;
    if (lease_table_put(&b->table, &e) != 0)
        return -1;
    b->active[b->active_len++] = e.key;
    b->granted++;
    return 0;
}

/* Hands back one of the active leases, chosen by the clients' behaviour. */
int broker_release_one(broker *b)
{
    if (b->active_len == 0)
        return -1;
    size_t pick = (size_t)(broker_rand(b) % b->active_len);
    uint64_t id = b->active[pick];
    b->active[pick] = b->active[--b->active_len];
    if (lease_table_remove(&b->table, id) != 0)
        return -1;
    b->released++;
    return 0;
}

/* One tick: a client asks for a connection slot and, once the broker holds
 * its working set of leases, another client hands one back. */
void broker_step(broker *b)
{
    b->clock++;
    uint32_t owner = (uint32_t)(broker_rand(b) % 512u);
    uint64_t ttl = 30u + broker_rand(b) % 600u;
    if (broker_grant(b, owner, ttl) != 0)
        return;
    if (b->active_len > b->window)
        broker_release_one(b);
}
