from . import formatting
from .builder import ReportBuilder
from .models import Response


class ReportService:
    """Answers report requests from the front end."""

    def __init__(self, store, currency="USD"):
        self._builder = ReportBuilder(store, currency)
        self.served = 0

    def handle(self, request):
        body = self._builder.render(request.account, request.month, request.style)
        self.served += 1
        return Response(request.account, request.month, body, formatting.checksum(body))
