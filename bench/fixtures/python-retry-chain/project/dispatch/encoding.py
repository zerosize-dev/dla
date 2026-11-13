import json
import zlib


def encode(delivery):
    """Serialise a delivery the way the receiving endpoints expect it."""
    document = {
        "id": delivery.delivery_id,
        "event": delivery.event,
        "data": delivery.fields,
    }
    return json.dumps(document, sort_keys=True, separators=(",", ":")).encode("utf-8")


def signature(body, secret):
    return zlib.crc32(body, zlib.crc32(secret.encode("utf-8"))) & 0xFFFFFFFF
