"""Which tenants were last read with the default decoder, for the admin page."""

_last_reason = {}


def record(tenant_name, reason):
    _last_reason[tenant_name] = reason


def report():
    return dict(sorted(_last_reason.items()))
