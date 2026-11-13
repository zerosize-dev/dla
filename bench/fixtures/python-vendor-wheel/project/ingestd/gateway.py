from . import decoder, sessions


class Gateway:
    def __init__(self, sink):
        self._sink = sink
        self.packets = 0
        self.readings = 0

    def handle(self, packet):
        frame, readings = decoder.decode(packet)
        sessions.remember(packet, frame)
        for reading in readings:
            self._sink.write(reading)
        self.packets += 1
        self.readings += len(readings)
        return len(readings)
