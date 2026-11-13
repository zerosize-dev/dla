import random

from .errors import TransportError


class SimulatedTransport:
    """Stands in for the HTTP client: some endpoints refuse requests now and then."""

    def __init__(self, failure_rate=0.55, seed=5):
        self._rng = random.Random(seed)
        self._failure_rate = failure_rate
        self.sent = 0

    def send(self, endpoint, body, signature):
        if self._rng.random() < self._failure_rate:
            raise TransportError(f"{endpoint} answered 503")
        self.sent += 1
        return len(body) + 4
