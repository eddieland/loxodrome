from __future__ import annotations

import math

import pytest

from geodist import BoundingBox, Point, Point3D, geodesic_distance, geodesic_distance_3d
from geodist.ops import hausdorff, hausdorff_directed, hausdorff_directed_clipped
from geodist import pure


def test_spherical_distance_matches_rust() -> None:
    origin = Point(37.6189, -122.3750)
    destination = Point(40.6413, -73.7781)

    rust_distance = geodesic_distance(origin, destination)
    pure_distance = pure.geodesic_distance_sphere(origin, destination)

    assert pure_distance == pytest.approx(rust_distance, rel=1e-9)


def test_bearings_match_round_trip() -> None:
    origin = Point(10.0, 20.0)
    destination = Point(11.0, 21.0)

    result = pure.geodesic_with_bearings_sphere(origin, destination)
    assert result.distance_m > 0
    assert 0.0 <= result.initial_bearing_deg < 360.0
    assert 0.0 <= result.final_bearing_deg < 360.0

    reverse = pure.geodesic_with_bearings_sphere(destination, origin)
    assert math.isclose(result.initial_bearing_deg, reverse.final_bearing_deg, rel_tol=1e-9)


def test_chord_distance_tracks_rust() -> None:
    origin = Point3D(37.6189, -122.3750, 10.0)
    destination = Point3D(40.6413, -73.7781, 20.0)

    rust_distance = geodesic_distance_3d(origin, destination)
    pure_distance = pure.geodesic_distance_3d_chord(origin, destination)
    assert pure_distance == pytest.approx(rust_distance, rel=3e-3)


def test_hausdorff_matches_rust() -> None:
    set_a = [Point(0.0, 0.0), Point(1.0, 1.0)]
    set_b = [Point(0.0, 1.0), Point(2.0, 2.0)]

    rust_directed = hausdorff_directed(set_a, set_b)
    pure_directed = pure.hausdorff_directed_naive(set_a, set_b)
    assert pure_directed.distance_m == pytest.approx(rust_directed.distance_m, rel=1e-9)
    assert pure_directed.origin_index == rust_directed.origin_index
    assert pure_directed.candidate_index == rust_directed.candidate_index

    rust_symmetric = hausdorff(set_a, set_b)
    pure_symmetric = pure.hausdorff_naive(set_a, set_b)
    assert pure_symmetric.distance_m == pytest.approx(rust_symmetric.distance_m, rel=1e-9)


def test_clipped_hausdorff_matches_rust() -> None:
    set_a = [Point(-0.5, -0.5), Point(5.0, 5.0)]
    set_b = [Point(-0.5, 0.5), Point(5.0, -5.0)]
    bbox = BoundingBox(-1.0, 1.0, -1.0, 1.0)

    rust_directed = hausdorff_directed_clipped(set_a, set_b, bbox)
    pure_directed = pure.hausdorff_directed_clipped_naive(set_a, set_b, bbox)
    assert pure_directed.distance_m == pytest.approx(rust_directed.distance_m, rel=1e-9)

    rust_symmetric = hausdorff(set_a, set_b)
    pure_symmetric = pure.hausdorff_naive(set_a, set_b)
    assert pure_symmetric.distance_m >= pure_directed.distance_m
    assert rust_symmetric.distance_m >= rust_directed.distance_m

