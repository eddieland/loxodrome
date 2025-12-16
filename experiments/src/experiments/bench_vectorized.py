"""Lightweight benchmarks for the vectorized geodesic helpers."""

from __future__ import annotations

import argparse
import time

import numpy as np

from loxodrome import ops, vectorized as vz
from loxodrome.geometry import Point


def _time_call(fn, *, repeat: int) -> float:
    best = float("inf")
    for _ in range(repeat):
        start = time.perf_counter()
        fn()
        best = min(best, time.perf_counter() - start)
    return best


def benchmark_pairwise(count: int, repeat: int) -> None:
    origins = np.column_stack((np.linspace(0.0, 45.0, count), np.linspace(0.0, 90.0, count)))
    destinations = np.column_stack((np.linspace(1.0, 46.0, count), np.linspace(1.0, 91.0, count)))

    origin_batch = vz.points_from_coords(origins)
    destination_batch = vz.points_from_coords(destinations)

    # Warmup to avoid import/compile noise.
    vz.geodesic_distance_batch(origin_batch, destination_batch)

    vectorized_best = _time_call(
        lambda: vz.geodesic_distance_batch(origin_batch, destination_batch),
        repeat=repeat,
    )

    scalar_sample = min(count, 5_000)
    scalar_pairs = [
        (Point(float(lat1), float(lon1)), Point(float(lat2), float(lon2)))
        for (lat1, lon1), (lat2, lon2) in zip(origins[:scalar_sample], destinations[:scalar_sample])
    ]
    scalar_best = _time_call(lambda: [ops.geodesic_distance(a, b) for a, b in scalar_pairs], repeat=repeat)

    print(f"Vectorized distance ({count} pairs): {vectorized_best * 1e3:.2f} ms")
    print(f"Scalar distance ({scalar_sample} pairs): {scalar_best * 1e3:.2f} ms")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, default=50_000, help="Number of pairs to benchmark")
    parser.add_argument("--repeat", type=int, default=3, help="Repetitions to collect best timings")
    args = parser.parse_args()
    benchmark_pairwise(count=args.count, repeat=args.repeat)


if __name__ == "__main__":
    main()
