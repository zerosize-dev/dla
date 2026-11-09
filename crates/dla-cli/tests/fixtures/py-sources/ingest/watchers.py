class CheckpointWatcher:
    """Tracks how far a batch is behind the newest checkpoint."""

    def __init__(self, batch_id, sensors, known_sensors):
        self.batch_id = batch_id
        self.label = f"batch-{batch_id:08d}"
        self.sensors = tuple(sorted(sensors))
        self.marks = dict.fromkeys(known_sensors, batch_id)
        self.lag = 0

    def __call__(self, event):
        self.lag = max(0, event["batch_id"] - self.batch_id)
        for sensor in self.sensors:
            self.marks[sensor] = event["batch_id"]
