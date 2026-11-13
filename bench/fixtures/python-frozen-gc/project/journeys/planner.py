from .models import Journey, Quote


def build(itinerary):
    journey = Journey(itinerary.reference)
    stops = itinerary.stops
    for a, b in zip(stops, stops[1:]):
        journey.add_leg(a, b, 40 + (a * 13 + b * 7) % 600)
    return journey


def price(journey, table, band):
    for leg in journey.legs:
        base = table.rate(leg.origin, leg.destination, band)
        surcharge = 3 if leg.previous is not None and not leg.previous.connects_to(leg) else 0
        leg.fare = base * (leg.distance // 20 + 1) + surcharge


def quote(itinerary, table, band):
    journey = build(itinerary)
    price(journey, table, band)
    return Quote(journey.reference, len(journey.legs), journey.total_distance(), journey.total_fare())
