from dataclasses import dataclass, field


@dataclass(frozen=True)
class Delivery:
    delivery_id: int
    endpoint: str
    event: str
    fields: dict = field(default_factory=dict)


@dataclass(frozen=True)
class Receipt:
    delivery_id: int
    endpoint: str
    attempts: int
    size: int
