#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "types.h"

#define STAGE_SIZE 4096u

struct spool {
    int fd;
    size_t used;
    char *stage;
};

static int flush_stage(spool *s)
{
    size_t done = 0;
    while (done < s->used) {
        ssize_t n = write(s->fd, s->stage + done, s->used - done);
        if (n < 0)
            return -1;
        done += (size_t)n;
    }
    s->used = 0;
    return 0;
}

spool *spool_open(const char *path)
{
    spool *s = malloc(sizeof *s);
    if (s == NULL)
        return NULL;
    s->stage = malloc(STAGE_SIZE);
    if (s->stage == NULL) {
        free(s);
        return NULL;
    }
    s->fd = open(path, O_WRONLY | O_CREAT | O_APPEND, 0600);
    if (s->fd < 0) {
        free(s->stage);
        free(s);
        return NULL;
    }
    s->used = 0;
    return s;
}

int spool_write(spool *s, const void *data, size_t len)
{
    const char *p = data;
    while (len > 0) {
        size_t room = STAGE_SIZE - s->used;
        size_t n = len < room ? len : room;
        memcpy(s->stage + s->used, p, n);
        s->used += n;
        p += n;
        len -= n;
        if (s->used == STAGE_SIZE && flush_stage(s) != 0)
            return -1;
    }
    return 0;
}

void spool_close(spool *s)
{
    if (s == NULL)
        return;
    flush_stage(s);
    close(s->fd);
    free(s->stage);
    free(s);
}
