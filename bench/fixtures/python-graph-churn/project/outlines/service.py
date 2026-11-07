from . import toc, tuning
from .models import OutlineResult
from .parser import parse


class OutlineService:
    """Turns submitted outlines into numbered tables of contents."""

    def __init__(self, max_depth=3):
        self._max_depth = max_depth
        self.converted = 0
        tuning.install()

    def convert(self, request):
        tree = parse(request.name, request.text)
        body = toc.render(tree, self._max_depth)
        self.converted += 1
        return OutlineResult(request.name, tree.size(), body, toc.checksum(body))
