class Record:
    """One parsed reading."""

    def __init__(self, timestamp, sensor, value, tags):
        self.timestamp = timestamp
        self.sensor = sensor
        self.value = value
        self.tags = tags
        self.delta = 0.0
        self.previous = None


class Summary:
    """Per-batch aggregate handed to the sinks."""

    def __init__(self, batch_id):
        self.batch_id = batch_id
        self.count = 0
        self.total = 0.0
        self.largest_jump = 0.0
        self.sensors = set()

    def add(self, record):
        self.count += 1
        self.total += record.value
        self.largest_jump = max(self.largest_jump, abs(record.delta))
        self.sensors.add(record.sensor)

    def mean(self):
        return self.total / self.count if self.count else 0.0
