from dataclasses import dataclass


@dataclass(frozen=True)
class Packet:
    packet_id: int
    device: str
    raw: bytes


@dataclass(frozen=True)
class Reading:
    packet_id: int
    device: str
    channel: int
    value: float
