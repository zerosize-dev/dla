from .models import Record

_newest = None


class ParseError(ValueError):
    pass


def parse_line(line):
    """Parse 'timestamp,sensor,value,tag;tag' into a Record.

    The delta of a record is measured against the record parsed just before it.
    """
    global _newest
    parts = line.strip().split(",")
    if len(parts) != 4:
        raise ParseError(f"expected 4 fields, got {len(parts)}: {line!r}")
    try:
        timestamp = int(parts[0])
        value = float(parts[2])
    except ValueError as exc:
        raise ParseError(str(exc)) from None
    tags = tuple(tag for tag in parts[3].split(";") if tag)
    record = Record(timestamp, parts[1], value, tags)
    if _newest is not None:
        record.previous = _newest
        record.delta = value - _newest.value
    _newest = record
    return record


def parse_batch(lines):
    records = []
    for line in lines:
        if line.strip():
            records.append(parse_line(line))
    return records
