from . import health
from .sender import Sender


class DeliveryService:
    def __init__(self, transport, secret="shared-secret"):
        self._sender = Sender(transport, secret)
        self.delivered = 0
        self.attempts = 0

    def handle(self, delivery):
        receipt = self._sender.deliver(delivery)
        self.delivered += 1
        self.attempts += receipt.attempts
        return receipt

    def status(self):
        return health.snapshot()
