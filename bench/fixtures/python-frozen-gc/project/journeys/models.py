from dataclasses import dataclass


class Leg:
    """One hop of a journey, linked to its neighbours and to the journey."""

    def __init__(self, journey, origin, destination, distance):
        self.journey = journey
        self.origin = origin
        self.destination = destination
        self.distance = distance
        self.previous = None
        self.following = None
        self.fare = 0

    def connects_to(self, other):
        return self.destination == other.origin


class Journey:
    """An ordered list of legs; the legs point back at it."""

    def __init__(self, reference):
        self.reference = reference
        self.legs = []

    def add_leg(self, origin, destination, distance):
        leg = Leg(self, origin, destination, distance)
        if self.legs:
            leg.previous = self.legs[-1]
            self.legs[-1].following = leg
        self.legs.append(leg)
        return leg

    def total_distance(self):
        return sum(leg.distance for leg in self.legs)

    def total_fare(self):
        return sum(leg.fare for leg in self.legs)


@dataclass(frozen=True)
class Itinerary:
    reference: int
    stops: tuple


@dataclass(frozen=True)
class Quote:
    reference: int
    legs: int
    distance: int
    fare: int
