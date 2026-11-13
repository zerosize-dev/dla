import unittest

from dispatch import health
from dispatch.errors import TransportError
from dispatch.load import deliveries, run
from dispatch.models import Delivery
from dispatch.service import DeliveryService
from dispatch.transport import SimulatedTransport


class ScriptedTransport:
    def __init__(self, failures):
        self.failures = failures
        self.calls = 0

    def send(self, endpoint, body, signature):
        self.calls += 1
        if self.calls <= self.failures:
            raise TransportError("503")
        return len(body)


def one(endpoint="https://hooks.example.test/a"):
    return Delivery(1, endpoint, "plan.changed", {"account": 7})


class DeliveryTests(unittest.TestCase):
    def setUp(self):
        health._last_failure.clear()
        health._last_success.clear()

    def test_clean_delivery_takes_one_attempt(self):
        service = DeliveryService(ScriptedTransport(0))
        self.assertEqual(service.handle(one()).attempts, 1)

    def test_retries_until_the_endpoint_accepts(self):
        service = DeliveryService(ScriptedTransport(3))
        self.assertEqual(service.handle(one()).attempts, 4)

    def test_status_reports_last_success_and_failure(self):
        service = DeliveryService(ScriptedTransport(2))
        service.handle(one())
        row = service.status()[0]
        self.assertEqual(row["last_ok"], 1)
        self.assertIn("attempt 2 failed", row["last_failure"])

    def test_status_is_empty_before_any_delivery(self):
        self.assertEqual(DeliveryService(ScriptedTransport(0)).status(), [])

    def test_load_is_deterministic(self):
        self.assertEqual(run(300)[1], run(300)[1])

    def test_every_delivery_is_accepted_in_the_end(self):
        service, _ = run(200)
        self.assertEqual(service.delivered, 200)
        self.assertGreater(service.attempts, 200)

    def test_workload_spreads_over_endpoints(self):
        self.assertEqual(len({d.endpoint for d in deliveries(60)}), 12)

    def test_failure_rate_is_configurable(self):
        service = DeliveryService(SimulatedTransport(failure_rate=0.0))
        service.handle(one())
        self.assertEqual(service.attempts, 1)


if __name__ == "__main__":
    unittest.main()
