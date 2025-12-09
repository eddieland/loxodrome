"""Benchmark the pure-Python kernels against the Rust-backed bindings."""

from __future__ import annotations

import argparse
import time

import numpy as np

from geodist import Point
from geodist import pure
from geodist import ops


def _time_call(fn, *, repeat: int) -> float:
    best = float("inf")
    for _ in range(repeat):
        start = time.perf_counter()
        fn()
        best = min(best, time.perf_counter() - start)
    return best


def benchmark_geodesic(count: int, repeat: int) -> None:
    origins = np.column_stack((np.linspace(0.0, 45.0, count), np.linspace(0.0, 90.0, count)))
    destinations = np.column_stack((np.linspace(1.0, 46.0, count), np.linspace(1.0, 91.0, count)))

    point_pairs = [
        (Point(float(lat1), float(lon1)), Point(float(lat2), float(lon2)))
        for (lat1, lon1), (lat2, lon2) in zip(origins, destinations)
    ]

    rust_best = _time_call(lambda: [ops.geodesic_distance(a, b) for a, b in point_pairs], repeat=repeat)
    pure_best = _time_call(lambda: [pure.geodesic_distance_sphere(a, b) for a, b in point_pairs], repeat=repeat)

    print(f"Rust geodesic ({count} pairs): {rust_best * 1e3:.2f} ms")
    print(f"Pure geodesic ({count} pairs): {pure_best * 1e3:.2f} ms")


def benchmark_hausdorff(count: int, repeat: int) -> None:
    points_a = [Point(float(lat), float(lon)) for lat, lon in zip(np.linspace(0.0, 10.0, count), np.linspace(0.0, 5.0, count))]
    points_b = [Point(float(lat), float(lon)) for lat, lon in zip(np.linspace(5.0, 15.0, count), np.linspace(2.5, 7.5, count))]

    rust_best = _time_call(lambda: ops.hausdorff_directed(points_a, points_b), repeat=repeat)
    pure_best = _time_call(lambda: pure.hausdorff_directed_naive(points_a, points_b), repeat=repeat)

    print(f"Rust Hausdorff directed ({count} points): {rust_best * 1e3:.2f} ms")
    print(f"Pure Hausdorff directed ({count} points): {pure_best * 1e3:.2f} ms")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, default=1_000, help="Number of point pairs per benchmark")
    parser.add_argument("--repeat", type=int, default=3, help="Repetitions to capture best timings")
    args = parser.parse_args()

    benchmark_geodesic(count=args.count, repeat=args.repeat)
    benchmark_hausdorff(count=max(10, args.count // 10), repeat=args.repeat)


if __name__ == "__main__":
    main()
