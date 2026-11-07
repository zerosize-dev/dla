#include <stdlib.h>
#include <string.h>

#include "session.h"

/* Brings samples into the range the sink expects (studio swing to full). */
static void normalise(uint8_t *px, size_t n)
{
    for (size_t i = 0; i < n; i++) {
        int v = ((int)px[i] - 16) * 255 / 219;
        px[i] = (uint8_t)(v < 0 ? 0 : v > 255 ? 255 : v);
    }
}

/* Called by the decoder once per finished frame. */
static void on_frame(const acme_frame *f, void *user)
{
    session *s = user;

    uint8_t *scratch = malloc(f->size);
    if (scratch == NULL) {
        s->errors++;
        return;
    }
    memcpy(scratch, f->data, f->size);
    normalise(scratch, f->size);
    sink_push(&s->out, scratch, f->size);
}

static void on_status(int code, void *user)
{
    session *s = user;
    if (code != ACME_STATUS_OK)
        s->errors++;
}

int session_open(session *s, int width, int height)
{
    memset(s, 0, sizeof *s);
    if (sink_init(&s->out, 32, 4096) != 0)
        return -1;

    acme_options opts = ACME_OPTIONS_DEFAULT;
    opts.width = width;
    opts.height = height;
    opts.on_frame = on_frame;
    opts.on_status = on_status;
    opts.user = s;

    s->codec = acme_codec_open(&opts);
    if (s->codec == NULL) {
        sink_close(&s->out);
        return -1;
    }
    return 0;
}

void session_close(session *s)
{
    if (s->codec != NULL)
        acme_codec_close(s->codec);
    s->codec = NULL;
    sink_close(&s->out);
}

int session_feed(session *s, const uint8_t *packet, size_t len)
{
    return acme_codec_feed(s->codec, packet, len);
}
