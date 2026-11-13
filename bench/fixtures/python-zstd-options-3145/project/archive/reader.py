from compression import zstd

from . import fallbacks, segments
from .options import decoder_options


class SegmentReader:
    """Reads compressed log segments with the decoder settings of their tenant."""

    def __init__(self):
        self.segments = 0
        self.lines = 0
        self.default_decoders = 0

    def _decoder(self, tenant):
        try:
            return zstd.ZstdDecompressor(options=decoder_options(tenant.overrides()))
        except (TypeError, OverflowError, ValueError):
            self.default_decoders += 1
            fallbacks.record(tenant.name, "options rejected")
            return zstd.ZstdDecompressor()

    def read(self, tenant, blob):
        data = self._decoder(tenant).decompress(blob)
        lines = segments.split_lines(data)
        self.segments += 1
        self.lines += len(lines)
        return segments.level_counts(lines)
