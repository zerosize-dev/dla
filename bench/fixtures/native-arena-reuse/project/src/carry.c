#include <stdlib.h>
#include <string.h>

#include "types.h"

void carry_init(carry *c)
{
    memset(c, 0, sizeof *c);
}

int carry_add(carry *c, uint32_t batch, uint32_t count, int64_t total, uint32_t digest)
{
    summary *s = malloc(sizeof *s);
    if (s == NULL)
        return -1;
    s->batch = batch;
    s->count = count;
    s->total_cents = total;
    s->digest = digest;

    free(c->slots[c->next]);
    if (c->slots[c->next] == NULL)
        c->held++;
    c->slots[c->next] = s;
    c->next = (c->next + 1) % CARRY_SLOTS;
    return 0;
}

size_t carry_held(const carry *c)
{
    return c->held;
}

void carry_clear(carry *c)
{
    for (size_t i = 0; i < CARRY_SLOTS; i++) {
        free(c->slots[i]);
        c->slots[i] = NULL;
    }
    c->held = 0;
    c->next = 0;
}
