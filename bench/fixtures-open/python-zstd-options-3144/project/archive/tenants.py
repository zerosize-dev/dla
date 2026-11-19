import json
from dataclasses import dataclass


@dataclass(frozen=True)
class Tenant:
    """A customer of the archive and the decoder overrides stored for it."""

    name: str
    overrides_json: str

    def overrides(self):
        return json.loads(self.overrides_json) if self.overrides_json else {}


def make_tenants(count):
    tenants = []
    for n in range(count):
        overrides = {"window_log_max": 20 + n % 5}
        if n % 3 != 2:
            # Tiered customers carry a ladder of window sizes, in bytes, per segment age.
            overrides["window_ladder"] = [(1 << 20) + 4096 * k + n for k in range(64)]
        tenants.append(Tenant(f"tenant-{n:03d}", json.dumps(overrides)))
    return tenants
