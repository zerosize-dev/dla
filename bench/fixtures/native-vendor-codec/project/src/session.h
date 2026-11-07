#ifndef SESSION_H
#define SESSION_H

#include <acme/codec.h>

#include "sink.h"

typedef struct {
    acme_codec *codec;
    sink out;
    unsigned long errors;
} session;

int session_open(session *s, int width, int height);
void session_close(session *s);

/* Feeds one packet to the decoder; decoded frames arrive in on_frame(). */
int session_feed(session *s, const uint8_t *packet, size_t len);

#endif
