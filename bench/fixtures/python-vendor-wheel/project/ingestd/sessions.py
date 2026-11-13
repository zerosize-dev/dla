"""Per-device decode state, so that delta-coded packets can be resolved."""

_open_frames = {}


def remember(packet, frame):
    _open_frames[packet.packet_id] = frame


def lookup(packet_id):
    return _open_frames.get(packet_id)


def count():
    return len(_open_frames)
