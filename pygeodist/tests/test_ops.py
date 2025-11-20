from __future__ import annotations

import importlib.util

import pytest
from pytest import approx

if importlib.util.find_spec("geodist._geodist_rs") is None:
    pytest.skip("Rust extension is not built; skipping distance checks.", allow_module_level=True)

from geodist import (
    BoundingBox,
    GeodesicResult,
    Point,
    Point3D,
    geodesic_distance,
    geodesic_distance_3d,
    geodesic_with_bearings,
    hausdorff,
    hausdorff_3d,
    hausdorff_clipped,
    hausdorff_clipped_3d,
    hausdorff_directed,
    hausdorff_directed_3d,
    hausdorff_directed_clipped,
    hausdorff_directed_clipped_3d,
)


def test_geodesic_distance_matches_rust_kernel() -> None:
    origin = Point(0.0, 0.0)
    east = Point(0.0, 1.0)

    assert geodesic_distance(origin, east) == approx(111_195.080_233_532_9)


def test_geodesic_distance_3d_matches_vertical_offset() -> None:
    ground = Point3D(0.0, 0.0, 0.0)
    elevated = Point3D(0.0, 0.0, 150.0)

    assert geodesic_distance_3d(ground, elevated) == approx(150.0)


def test_geodesic_with_bearings_returns_distance_and_angles() -> None:
    origin = Point(0.0, 0.0)
    east = Point(0.0, 1.0)

    result = geodesic_with_bearings(origin, east)
    assert isinstance(result, GeodesicResult)
    assert result.distance_meters == approx(111_195.080_233_532_9)
    assert result.initial_bearing_degrees == approx(90.0)
    assert result.final_bearing_degrees == approx(90.0)


def test_hausdorff_and_directed_match_expected_distances() -> None:
    origin = Point(0.0, 0.0)
    east = Point(0.0, 1.0)

    assert hausdorff([origin], [east]) == approx(geodesic_distance(origin, east))
    assert hausdorff_directed([origin], [east]) == approx(geodesic_distance(origin, east))


def test_hausdorff_clipped_filters_points() -> None:
    origin = Point(0.0, 0.0)
    east = Point(0.0, 1.0)
    east_only_box = BoundingBox(-1.0, 1.0, 0.5, 1.5)

    # Without clipping, A includes the origin so the maximum mismatch is origin->east.
    assert hausdorff_directed([origin, east], [east]) == approx(geodesic_distance(origin, east))

    # Clipping removes the origin, so both sets reduce to [east] and the distance collapses to 0.
    assert hausdorff_directed_clipped([origin, east], [east], east_only_box) == approx(0.0)
    assert hausdorff_clipped([origin, east], [east], east_only_box) == approx(0.0)


def test_hausdorff_3d_matches_vertical_delta() -> None:
    ground = Point3D(0.0, 0.0, 0.0)
    elevated = Point3D(0.0, 0.0, 200.0)

    assert hausdorff_directed_3d([ground], [elevated]) == approx(200.0)
    assert hausdorff_3d([ground], [elevated]) == approx(200.0)


def test_hausdorff_3d_clipped_filters_points() -> None:
    inside = Point3D(0.0, 0.0, 50.0)
    outside = Point3D(10.0, 0.0, 0.0)
    box = BoundingBox(-1.0, 1.0, -1.0, 1.0)

    assert hausdorff_clipped_3d([inside, outside], [inside], box) == approx(0.0)
    assert hausdorff_directed_clipped_3d([inside, outside], [inside], box) == approx(0.0)
