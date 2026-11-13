import unittest

from journeys import planner
from journeys.fares import FareTable
from journeys.load import itineraries, run
from journeys.models import Itinerary, Journey
from journeys.worker import BatchWorker


class JourneyTests(unittest.TestCase):
    def test_legs_link_to_their_neighbours(self):
        journey = planner.build(Itinerary(1, (1, 2, 3, 4)))
        self.assertEqual(len(journey.legs), 3)
        self.assertIs(journey.legs[1].previous, journey.legs[0])
        self.assertIs(journey.legs[1].following, journey.legs[2])
        self.assertIs(journey.legs[2].journey, journey)

    def test_distance_is_the_sum_of_the_legs(self):
        journey = Journey(9)
        journey.add_leg(1, 2, 100)
        journey.add_leg(2, 3, 50)
        self.assertEqual(journey.total_distance(), 150)

    def test_quote_prices_every_leg(self):
        quote = planner.quote(Itinerary(5, (1, 2, 3)), FareTable(4), 0)
        self.assertEqual(quote.legs, 2)
        self.assertGreater(quote.fare, 0)

    def test_fare_table_wraps_zone_numbers(self):
        table = FareTable(6)
        self.assertEqual(table.rate(1, 2, 3), table.rate(7, 8, 15))

    def test_worker_counts_batches(self):
        worker, _ = run(100, batch=10)
        self.assertEqual((worker.priced, worker.batches), (100, 10))

    def test_short_final_batch_is_priced(self):
        worker, _ = run(26, batch=25)
        self.assertEqual((worker.priced, worker.batches), (26, 2))

    def test_run_is_deterministic(self):
        self.assertEqual(run(120)[1], run(120)[1])

    def test_generator_yields_the_requested_count(self):
        self.assertEqual(len(list(itineraries(17))), 17)


if __name__ == "__main__":
    unittest.main()
