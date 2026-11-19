import random
import unittest
from compression import zstd

from archive import fallbacks
from archive.options import decoder_options
from archive.reader import SegmentReader
from archive.segments import level_counts, make_segment
from archive.tenants import Tenant, make_tenants
from archive.load import run


class ArchiveTests(unittest.TestCase):
    def test_segments_round_trip(self):
        blob = make_segment(random.Random(1), lines=10)
        self.assertEqual(len(zstd.decompress(blob).split(b"\n")), 10)

    def test_level_counts_add_up(self):
        counts = level_counts(["a INFO x", "a WARN x", "a INFO x"])
        self.assertEqual(counts, {"INFO": 2, "WARN": 1})

    def test_known_override_becomes_a_parameter(self):
        options = decoder_options({"window_log_max": 21})
        self.assertEqual(list(options.values()), [21])
        self.assertNotIn("window_log_max", options)

    def test_unknown_override_keeps_its_name(self):
        self.assertIn("window_ladder", decoder_options({"window_ladder": [1]}))

    def test_tenant_without_overrides_reads_normally(self):
        reader = SegmentReader()
        blob = make_segment(random.Random(2), lines=5)
        self.assertEqual(sum(reader.read(Tenant("t", ""), blob).values()), 5)
        self.assertEqual(reader.default_decoders, 0)

    def test_tenant_with_unsupported_override_falls_back(self):
        reader = SegmentReader()
        blob = make_segment(random.Random(3), lines=5)
        tenant = make_tenants(1)[0]
        self.assertEqual(sum(reader.read(tenant, blob).values()), 5)
        self.assertEqual(reader.default_decoders, 1)

    def test_tenants_cycle_through_the_ladder_rule(self):
        tenants = make_tenants(6)
        self.assertIn("window_ladder", tenants[0].overrides())
        self.assertNotIn("window_ladder", tenants[2].overrides())

    def test_fallbacks_are_reported_per_tenant(self):
        reader = SegmentReader()
        blob = make_segment(random.Random(4), lines=5)
        tenant = make_tenants(1)[0]
        reader.read(tenant, blob)
        self.assertEqual(fallbacks.report()[tenant.name], "options rejected")

    def test_load_reads_every_segment(self):
        reader, _ = run(90)
        self.assertEqual(reader.segments, 90)


if __name__ == "__main__":
    unittest.main()
