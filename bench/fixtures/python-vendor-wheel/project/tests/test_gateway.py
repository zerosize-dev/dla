import unittest

from ingestd import sessions
from ingestd.gateway import Gateway
from ingestd.load import packets
from ingestd.sink import ListSink


class GatewayTests(unittest.TestCase):
    def setUp(self):
        sessions._open_frames.clear()

    def test_every_packet_yields_readings(self):
        gateway = Gateway(ListSink())
        for packet in packets(10):
            self.assertGreater(gateway.handle(packet), 0)

    def test_sink_keeps_its_window(self):
        sink = ListSink(limit=5)
        gateway = Gateway(sink)
        for packet in packets(20):
            gateway.handle(packet)
        self.assertEqual(len(sink.drain()), 5)

    def test_drain_empties_the_buffer(self):
        sink = ListSink()
        Gateway(sink).handle(next(packets(1)))
        sink.drain()
        self.assertEqual(sink.drain(), [])

    def test_frames_are_remembered_by_packet_id(self):
        gateway = Gateway(ListSink())
        for packet in packets(3):
            gateway.handle(packet)
        self.assertEqual(sessions.count(), 3)
        self.assertIsNotNone(sessions.lookup(1))

    def test_unknown_packet_has_no_frame(self):
        self.assertIsNone(sessions.lookup(99))

    def test_generator_is_deterministic(self):
        self.assertEqual(list(packets(5)), list(packets(5)))


if __name__ == "__main__":
    unittest.main()
