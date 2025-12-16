from __future__ import annotations

import math

import pytest

from loxodrome import BoundingBox, Ellipsoid, InvalidGeometryError, LineString, Point, Point3D
from loxodrome.geometry import Polygon


def test_point_accepts_numeric_coordinates() -> None:
    point = Point(12.5, -45)

    assert point.lat == 12.5
    assert point.lon == -45.0
    assert tuple(point) == (12.5, -45.0)
    assert point.to_tuple() == (12.5, -45.0)


@pytest.mark.parametrize(
    ("latitude", "longitude"),
    [
        (91.0, 0.0),
        (-91.0, 0.0),
        (0.0, 181.0),
        (0.0, -181.0),
    ],
)
def test_point_rejects_out_of_range_coordinates(latitude: float, longitude: float) -> None:
    with pytest.raises(InvalidGeometryError):
        Point(latitude, longitude)


@pytest.mark.parametrize(
    ("latitude", "longitude"),
    [
        (math.nan, 0.0),
        (0.0, math.nan),
        (math.inf, 0.0),
        (0.0, -math.inf),
    ],
)
def test_point_rejects_non_finite(latitude: float, longitude: float) -> None:
    with pytest.raises(InvalidGeometryError):
        Point(latitude, longitude)


def test_point_rejects_bool_inputs() -> None:
    with pytest.raises(InvalidGeometryError):
        Point(True, 0.0)

    with pytest.raises(InvalidGeometryError):
        Point(0.0, False)


def test_ellipsoid_accepts_axes_and_wgs84_factory() -> None:
    ellipsoid = Ellipsoid(6_378_137.0, 6_356_752.314_245)
    assert ellipsoid.to_tuple() == (6_378_137.0, 6_356_752.314_245)
    assert Ellipsoid.wgs84() == ellipsoid


@pytest.mark.parametrize(
    ("semi_major", "semi_minor"),
    [
        (math.nan, 6_300_000.0),
        (6_300_000.0, math.inf),
        (0.0, 6_300_000.0),
        (6_300_000.0, -10.0),
        (6_300_000.0, 6_500_000.0),
    ],
)
def test_ellipsoid_rejects_invalid_axes(semi_major: float, semi_minor: float) -> None:
    with pytest.raises(InvalidGeometryError):
        Ellipsoid(semi_major, semi_minor)


def test_point3d_accepts_altitude_and_matches_tuple() -> None:
    point = Point3D(12.5, -45.0, 250.0)

    assert point.lat == 12.5
    assert point.lon == -45.0
    assert point.altitude_m == 250.0
    assert tuple(point) == (12.5, -45.0, 250.0)
    assert point.to_tuple() == (12.5, -45.0, 250.0)


def test_point3d_rejects_non_finite_altitude() -> None:
    with pytest.raises(InvalidGeometryError):
        Point3D(0.0, 0.0, math.nan)

    with pytest.raises(InvalidGeometryError):
        Point3D(0.0, 0.0, math.inf)


def test_point3d_rejects_bool_inputs() -> None:
    with pytest.raises(InvalidGeometryError):
        Point3D(0.0, 0.0, True)


def test_bounding_box_accepts_ordered_coordinates() -> None:
    bbox = BoundingBox(-10.0, 10.0, -20.0, 20.0)
    assert bbox.to_tuple() == (-10.0, 10.0, -20.0, 20.0)


def test_bounding_box_accepts_antimeridian_wrap() -> None:
    bbox = BoundingBox(-5.0, 5.0, 170.0, -170.0)
    assert bbox.to_tuple() == (-5.0, 5.0, 170.0, -170.0)


def test_bounding_box_rejects_invalid_ranges() -> None:
    with pytest.raises(InvalidGeometryError):
        BoundingBox(10.0, -10.0, -20.0, 20.0)


def test_bounding_box_rejects_bool_inputs() -> None:
    with pytest.raises(InvalidGeometryError):
        BoundingBox(True, 10.0, -20.0, 20.0)


def test_polygon_accepts_exterior_and_hole() -> None:
    exterior = [
        (0.0, 0.0),
        (0.0, 1.0),
        (1.0, 1.0),
        (1.0, 0.0),
        (0.0, 0.0),
    ]
    hole = [
        (0.2, 0.2),
        (0.4, 0.2),
        (0.4, 0.4),
        (0.2, 0.4),
        (0.2, 0.2),
    ]

    polygon = Polygon(exterior, [hole])
    exterior_out, holes_out = polygon.to_tuple()
    assert len(exterior_out) == 5
    assert len(holes_out) == 1


def test_linestring_accepts_vertices_and_to_tuple() -> None:
    line = LineString([(0.0, 0.0), (0.0, 1.0)])
    assert len(line) == 2
    assert line.to_tuple() == [(0.0, 0.0), (0.0, 1.0)]


def test_linestring_rejects_degenerate_after_dedup() -> None:
    with pytest.raises(InvalidGeometryError):
        LineString([(0.0, 0.0), (0.0, 0.0)])


def test_linestring_densify_returns_expected_samples() -> None:
    start = (0.0, 0.0)
    end = (0.0, 0.0899)
    line = LineString([start, end])

    samples = line.densify()
    assert len(samples) == 101
    assert samples[0].to_tuple() == start
    assert samples[-1].to_tuple() == pytest.approx(end)
