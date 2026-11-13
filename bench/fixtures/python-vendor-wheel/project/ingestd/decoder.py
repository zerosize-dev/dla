import tessera_wire

from .models import Reading

_SCHEMA = tessera_wire.Schema.load("telemetry-v4")


def decode(packet):
    """Decode a packet into readings with the vendor's wire library."""
    frame = tessera_wire.parse(packet.raw, _SCHEMA)
    return frame, [Reading(packet.packet_id, packet.device, ch, val) for ch, val in frame.samples()]
