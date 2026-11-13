import random
import unittest

from statements import ledger
from statements.importer import RECENT, Importer
from statements.load import batches, make_lines, run
from statements.parser import parse_line


class StatementTests(unittest.TestCase):
    def test_parse_line_splits_the_fields(self):
        row = parse_line("12|2024-03-05|-2500|card payment|a,b")
        self.assertEqual(row["account"], 12)
        self.assertEqual(row["cents"], -2500)
        self.assertEqual(row["tags"], ("a", "b"))

    def test_empty_tags_are_dropped(self):
        self.assertEqual(parse_line("1|2024-01-01|5|x|,a,")["tags"], ("a",))

    def test_totals_group_by_account(self):
        rows = [parse_line("1|d|100|m|"), parse_line("1|d|50|m|"), parse_line("2|d|7|m|")]
        totals = ledger.totals_by_account(rows)
        self.assertEqual([(t.account, t.count, t.total_cents) for t in totals],
                         [(1, 2, 150), (2, 1, 7)])

    def test_flagged_uses_the_absolute_amount(self):
        rows = [parse_line("1|d|-600|m|"), parse_line("1|d|10|m|")]
        self.assertEqual(len(ledger.flagged(rows, 500)), 1)

    def test_import_counts_rows(self):
        importer, _ = run(20)
        self.assertEqual(importer.batches, 20)
        self.assertGreater(importer.rows, 20 * 60 - 1)

    def test_recent_keeps_the_last_summaries_only(self):
        importer, _ = run(RECENT + 10)
        recent = importer.recent()
        self.assertEqual(len(recent), RECENT)
        self.assertEqual(recent[-1].batch_id, RECENT + 9)

    def test_batches_are_deterministic(self):
        self.assertEqual(list(batches(3)), list(batches(3)))

    def test_lines_have_five_fields(self):
        line = make_lines(random.Random(1), 1)[0]
        self.assertEqual(line.count("|"), 4)
        self.assertEqual(Importer().import_batch(0, [line]).rows, 1)


if __name__ == "__main__":
    unittest.main()
