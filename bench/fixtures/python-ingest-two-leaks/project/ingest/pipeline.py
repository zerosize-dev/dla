from . import registry
from .parser import parse_batch
from .models import Summary
from .watchers import CheckpointWatcher

CHECKPOINT_EVERY = 250


class Pipeline:
    """Parses incoming batches, aggregates them and tracks checkpoints."""

    def __init__(self):
        self.batches = 0
        self.rows = 0
        self.per_sensor = {}
        self.last_checkpoint = 0

    def process_batch(self, lines):
        self.batches += 1
        batch_id = self.batches
        records = parse_batch(lines)
        summary = Summary(batch_id)
        for record in records:
            summary.add(record)
            self.per_sensor[record.sensor] = record.value
        self.rows += summary.count
        watcher = CheckpointWatcher(batch_id, summary.sensors, self.per_sensor)
        registry.subscribe("checkpoint", watcher)
        if batch_id % CHECKPOINT_EVERY == 0:
            self.last_checkpoint = batch_id
            registry.fire("checkpoint", {"batch_id": batch_id})
        return summary
