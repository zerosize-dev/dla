import unittest

from reports import LedgerStore, ReportRequest, ReportService
from reports import formatting, load


class FormattingTests(unittest.TestCase):
    def test_money(self):
        self.assertEqual(formatting.money(123456), "$1,234.56")
        self.assertEqual(formatting.money(-5, "EUR"), "-EUR 0.05")

    def test_pad(self):
        self.assertEqual(formatting.pad("abc", 5), "abc  ")
        self.assertEqual(formatting.pad("abcdef", 4), "abcd")


class StoreTests(unittest.TestCase):
    def test_rows_are_stable(self):
        store = LedgerStore(rows_per_month=5)
        self.assertEqual(store.fetch(7, "2024-03"), store.fetch(7, "2024-03"))
        self.assertEqual(len(store.fetch(7, "2024-03")), 5)


class ServiceTests(unittest.TestCase):
    def setUp(self):
        self.service = ReportService(LedgerStore(rows_per_month=6))

    def test_summary_lists_categories_once(self):
        response = self.service.handle(ReportRequest(12, "2024-01"))
        body_lines = response.body.splitlines()[2:]
        names = [line.split()[0] for line in body_lines]
        self.assertEqual(len(names), len(set(names)))

    def test_detail_lists_every_row(self):
        response = self.service.handle(ReportRequest(12, "2024-01", "detail"))
        self.assertEqual(len(response.body.splitlines()), 2 + 6)

    def test_same_request_same_checksum(self):
        first = self.service.handle(ReportRequest(3, "2024-02"))
        second = self.service.handle(ReportRequest(3, "2024-02"))
        self.assertEqual(first.checksum, second.checksum)

    def test_load_counts_requests(self):
        service, _ = load.run(25, ReportService(LedgerStore(rows_per_month=4)))
        self.assertEqual(service.served, 25)


if __name__ == "__main__":
    unittest.main()
