"""What the status page shows for each subscriber endpoint."""

_last_failure = {}
_last_success = {}


def record_failure(endpoint, error):
    _last_failure[endpoint] = error


def record_success(endpoint, delivery_id):
    _last_success[endpoint] = delivery_id


def last_failure(endpoint):
    return _last_failure.get(endpoint)


def snapshot():
    rows = []
    for endpoint in sorted(set(_last_failure) | set(_last_success)):
        failure = _last_failure.get(endpoint)
        rows.append({
            "endpoint": endpoint,
            "last_ok": _last_success.get(endpoint),
            "last_failure": str(failure) if failure is not None else None,
        })
    return rows
