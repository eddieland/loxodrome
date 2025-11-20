from __future__ import annotations

import importlib.util

import pytest
from pytest import approx

if importlib.util.find_spec("geodist._geodist_rs") is None:
    pytest.skip("Rust extension is not built; skipping distance checks.", allow_module_level=True)

from geodist import Point, geodesic_distance


def test_geodesic_distance_matches_rust_kernel() -> None:
    origin = Point(0.0, 0.0)
    east = Point(0.0, 1.0)

    assert geodesic_distance(origin, east) == approx(111_195.080_233_532_9)


def test_geodesic_distance_requires_point_instances() -> None:
    with pytest.raises(TypeError):
        geodesic_distance((0.0, 0.0), (0.0, 1.0))  # type: ignore[arg-type]
