import unittest

from outlines import OutlineRequest, OutlineService, load, toc
from outlines.parser import OutlineError, parse

SAMPLE = """\
- intro
- design
  - storage
    - formats
  - network
- ops
"""


class ParserTests(unittest.TestCase):
    def test_builds_tree(self):
        tree = parse("sample", SAMPLE)
        self.assertEqual(tree.size(), 6)
        self.assertIn("design/storage/formats", tree.by_path)

    def test_parent_links(self):
        tree = parse("sample", SAMPLE)
        node = tree.by_path["design/network"]
        self.assertEqual(node.parent.title, "design")
        self.assertIs(node.tree, tree)

    def test_rejects_skipped_level(self):
        with self.assertRaises(OutlineError):
            parse("bad", "- a\n    - b\n")

    def test_rejects_odd_indent(self):
        with self.assertRaises(OutlineError):
            parse("bad", "- a\n   - b\n")


class TocTests(unittest.TestCase):
    def test_numbering(self):
        labels = [label for label, _ in toc.number_headings(parse("s", SAMPLE))]
        self.assertEqual(labels, ["1", "2", "2.1", "2.1.1", "2.2", "3"])

    def test_depth_limit(self):
        text = toc.render(parse("s", SAMPLE), max_depth=2)
        self.assertNotIn("formats", text)


class ServiceTests(unittest.TestCase):
    def test_convert(self):
        result = OutlineService().convert(OutlineRequest("sample", SAMPLE))
        self.assertEqual(result.headings, 6)
        self.assertTrue(result.toc.startswith("SAMPLE"))

    def test_load_counts(self):
        service, _ = load.run(20)
        self.assertEqual(service.converted, 20)


if __name__ == "__main__":
    unittest.main()
