#ifndef FRAME_H
#define FRAME_H

#include <stddef.h>
#include <stdint.h>

/* Wire layout of one frame (all integers little-endian):
 *
 *   offset  size  field
 *        0     2  magic, "FR"
 *        2     1  version
 *        3     1  flags
 *        4     4  sequence number
 *        8     4  decoded length
 *       12     4  encoded length
 *       16     n  run-length encoded payload
 *     16+n     4  CRC-32 of the decoded payload
 */
#define FRAME_MAGIC0 'F'
#define FRAME_MAGIC1 'R'
#define FRAME_VERSION 1
#define FRAME_HEADER_SIZE 16
#define FRAME_TRAILER_SIZE 4
#define FRAME_MAX_DECODED (64u * 1024u)

typedef enum {
    FRAME_OK = 0,
    FRAME_ERR_SHORT,
    FRAME_ERR_MAGIC,
    FRAME_ERR_VERSION,
    FRAME_ERR_SIZE,
    FRAME_ERR_PAYLOAD,
    FRAME_ERR_CHECKSUM
} frame_status;

typedef struct {
    uint32_t seq;
    uint8_t flags;
    uint8_t *pixels;   /* owned by the caller once decode_frame succeeds */
    size_t len;
} decoded_frame;

/* checksum.c */
uint32_t crc32_update(uint32_t crc, const uint8_t *data, size_t len);
uint32_t crc32_of(const uint8_t *data, size_t len);

/* rle.c */
size_t rle_encode(const uint8_t *in, size_t n, uint8_t *out, size_t cap);
int rle_expand(const uint8_t *in, size_t n, uint8_t *out, size_t want);

/* decoder.c */
frame_status decode_frame(const uint8_t *wire, size_t n, decoded_frame *out);
void frame_release(decoded_frame *f);
const char *frame_status_name(frame_status s);

/* synth.c: deterministic frame source used by the workload and the tests */
typedef struct {
    uint64_t state;
    uint32_t next_seq;
    uint32_t damage_every;   /* every n-th frame carries a damaged trailer; 0 = none */
} synth_source;

void synth_init(synth_source *s, uint64_t seed, uint32_t damage_every);
size_t synth_next(synth_source *s, uint8_t *wire, size_t cap);
uint32_t synth_rand(synth_source *s);

#endif
