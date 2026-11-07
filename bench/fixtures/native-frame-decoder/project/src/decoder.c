#include <stdlib.h>
#include <string.h>

#include "frame.h"

static uint32_t load_u32(const uint8_t *p)
{
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) |
           ((uint32_t)p[3] << 24);
}

frame_status decode_frame(const uint8_t *wire, size_t n, decoded_frame *out)
{
    if (n < FRAME_HEADER_SIZE + FRAME_TRAILER_SIZE)
        return FRAME_ERR_SHORT;
    if (wire[0] != FRAME_MAGIC0 || wire[1] != FRAME_MAGIC1)
        return FRAME_ERR_MAGIC;
    if (wire[2] != FRAME_VERSION)
        return FRAME_ERR_VERSION;

    uint32_t seq = load_u32(wire + 4);
    uint32_t raw_len = load_u32(wire + 8);
    uint32_t enc_len = load_u32(wire + 12);
    if (raw_len == 0 || raw_len > FRAME_MAX_DECODED)
        return FRAME_ERR_SIZE;
    if ((size_t)enc_len + FRAME_HEADER_SIZE + FRAME_TRAILER_SIZE > n)
        return FRAME_ERR_SIZE;

    uint8_t *buf = malloc(raw_len);
    if (buf == NULL)
        return FRAME_ERR_SIZE;

    if (rle_expand(wire + FRAME_HEADER_SIZE, enc_len, buf, raw_len) != 0) {
        free(buf);
        return FRAME_ERR_PAYLOAD;
    }

    uint32_t stored = load_u32(wire + FRAME_HEADER_SIZE + enc_len);
    if (crc32_of(buf, raw_len) != stored)
        return FRAME_ERR_CHECKSUM;

    out->seq = seq;
    out->flags = wire[3];
    out->pixels = buf;
    out->len = raw_len;
    return FRAME_OK;
}

void frame_release(decoded_frame *f)
{
    free(f->pixels);
    memset(f, 0, sizeof *f);
}

const char *frame_status_name(frame_status s)
{
    switch (s) {
    case FRAME_OK:           return "ok";
    case FRAME_ERR_SHORT:    return "short";
    case FRAME_ERR_MAGIC:    return "magic";
    case FRAME_ERR_VERSION:  return "version";
    case FRAME_ERR_SIZE:     return "size";
    case FRAME_ERR_PAYLOAD:  return "payload";
    case FRAME_ERR_CHECKSUM: return "checksum";
    }
    return "unknown";
}
