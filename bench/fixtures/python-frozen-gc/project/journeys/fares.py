"""The fare table the pricing workers load at start-up."""

ZONES = 24


class FareTable:
    def __init__(self, zones=ZONES):
        self._rates = {}
        for a in range(zones):
            for b in range(zones):
                self._rates[(a, b)] = tuple(1 + ((a * 31 + b * 17 + k) % 9) for k in range(12))
        self.zones = zones

    def rate(self, origin, destination, band):
        key = (origin % self.zones, destination % self.zones)
        return self._rates[key][band % 12]
