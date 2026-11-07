import unittest

from ingest import Pipeline, load, registry
from ingest.parser import ParseError, parse_batch, parse_line


class ParserTests(unittest.TestCase):
    def test_parse_line(self):
        record = parse_line("100,probe-001,21.5,indoor;north")
        self.assertEqual(record.timestamp, 100)
        self.assertEqual(record.sensor, "probe-001")
        self.assertEqual(sorted(record.tags), ["indoor", "north"])

    def test_delta_uses_previous_record(self):
        first, second = parse_batch(["1,a,10.0,x", "2,a,13.5,x"])
        self.assertAlmostEqual(second.delta, 3.5)
        self.assertIs(second.previous, first)

    def test_bad_field_count(self):
        with self.assertRaises(ParseError):
            parse_line("1,a,2.0")

    def test_bad_number(self):
        with self.assertRaises(ParseError):
            parse_line("x,a,2.0,t")

    def test_blank_lines_skipped(self):
        self.assertEqual(len(parse_batch(["", "1,a,1.0,t", "  "])), 1)


class PipelineTests(unittest.TestCase):
    def setUp(self):
        registry.reset()

    def test_summary(self):
        summary = Pipeline().process_batch(["1,a,2.0,t", "2,b,4.0,t"])
        self.assertEqual(summary.count, 2)
        self.assertAlmostEqual(summary.mean(), 3.0)
        self.assertEqual(summary.sensors, {"a", "b"})

    def test_checkpoint_reaches_watchers(self):
        pipeline = Pipeline()
        for lines in load.make_batches(250):
            pipeline.process_batch(lines)
        self.assertEqual(pipeline.last_checkpoint, 250)
        self.assertEqual(registry.listener_count("checkpoint"), 250)

    def test_load_counts_rows(self):
        pipeline = load.run(10)
        self.assertEqual(pipeline.rows, 10 * load.LINES_PER_BATCH)


if __name__ == "__main__":
    unittest.main()
