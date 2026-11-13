import time

from . import encoding, health
from .errors import RetryError, TransportError
from .models import Receipt


class Sender:
    """Delivers one event to one endpoint, retrying until the endpoint accepts it."""

    def __init__(self, transport, secret, pause=0.0):
        self._transport = transport
        self._secret = secret
        self._pause = pause

    def deliver(self, delivery):
        body = encoding.encode(delivery)
        signature = encoding.signature(body, self._secret)
        previous = health.last_failure(delivery.endpoint)
        attempt = 0
        while True:
            attempt += 1
            try:
                size = self._transport.send(delivery.endpoint, body, signature)
            except TransportError as exc:
                try:
                    raise RetryError(delivery.endpoint, attempt) from (previous or exc)
                except RetryError as failure:
                    previous = failure
                    health.record_failure(delivery.endpoint, failure)
                if self._pause:
                    time.sleep(self._pause)
                continue
            health.record_success(delivery.endpoint, delivery.delivery_id)
            return Receipt(delivery.delivery_id, delivery.endpoint, attempt, size)
