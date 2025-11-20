from __future__ import annotations

import importlib.util
import math

import pytest

if importlib.util.find_spec("geodist._geodist_rs") is None:
    pytest.skip("Rust extension is not built; skipping geometry checks.", allow_module_level=True)

from geodist import BoundingBox, InvalidGeometryError, Point, Point3D


def test_point_accepts_numeric_coordinates() -> None:
    point = Point(12.5, -45)

    assert point.latitude_degrees == 12.5
    assert point.longitude_degrees == -45.0
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


def test_point3d_accepts_altitude_and_matches_tuple() -> None:
    point = Point3D(12.5, -45.0, 250.0)

    assert point.latitude_degrees == 12.5
    assert point.longitude_degrees == -45.0
    assert point.altitude_meters == 250.0
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


def test_bounding_box_rejects_invalid_ranges() -> None:
    with pytest.raises(InvalidGeometryError):
        BoundingBox(10.0, -10.0, -20.0, 20.0)

    with pytest.raises(InvalidGeometryError):
        BoundingBox(-10.0, 10.0, 20.0, -20.0)


def test_bounding_box_rejects_bool_inputs() -> None:
    with pytest.raises(InvalidGeometryError):
        BoundingBox(True, 10.0, -20.0, 20.0)
