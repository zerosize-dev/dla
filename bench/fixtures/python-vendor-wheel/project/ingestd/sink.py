class ListSink:
    """Collects the readings of the current window until the caller drains it."""

    def __init__(self, limit=4096):
        self._limit = limit
        self._buffer = []

    def write(self, reading):
        self._buffer.append(reading)
        if len(self._buffer) > self._limit:
            del self._buffer[: len(self._buffer) - self._limit]

    def drain(self):
        items, self._buffer = self._buffer, []
        return items
