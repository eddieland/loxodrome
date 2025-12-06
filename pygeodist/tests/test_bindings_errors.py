"""Regression tests for Rust -> Python GeodistError mappings."""

from __future__ import annotations

import math

import pytest

from geodist import _geodist_rs as rs


@pytest.fixture()
def valid_points() -> tuple[rs.Point, rs.Point]:
    return rs.Point(0.0, 0.0), rs.Point(1.0, 1.0)


def test_invalid_latitude_maps_to_python_error(valid_points: tuple[rs.Point, rs.Point]) -> None:
    _, destination = valid_points

    with pytest.raises(rs.InvalidLatitudeError):
        rs.geodesic_distance(rs.Point(95.0, 0.0), destination)


def test_invalid_longitude_maps_to_python_error(valid_points: tuple[rs.Point, rs.Point]) -> None:
    origin, _ = valid_points

    with pytest.raises(rs.InvalidLongitudeError):
        rs.geodesic_distance(origin, rs.Point(0.0, 200.0))


def test_invalid_altitude_maps_to_python_error(valid_points: tuple[rs.Point, rs.Point]) -> None:
    origin, destination = valid_points

    with pytest.raises(rs.InvalidAltitudeError):
        rs.geodesic_distance_3d(
            rs.Point3D(origin.lat, origin.lon, math.inf),
            rs.Point3D(destination.lat, destination.lon, 0.0),
        )


def test_invalid_radius_maps_to_python_error(valid_points: tuple[rs.Point, rs.Point]) -> None:
    origin, destination = valid_points

    with pytest.raises(rs.InvalidRadiusError):
        rs.geodesic_distance_on_ellipsoid(origin, destination, rs.Ellipsoid(-1.0, 1.0))


def test_invalid_ellipsoid_axis_order_maps_to_python_error(valid_points: tuple[rs.Point, rs.Point]) -> None:
    origin, destination = valid_points

    with pytest.raises(rs.InvalidEllipsoidError):
        rs.geodesic_distance_on_ellipsoid(origin, destination, rs.Ellipsoid(1.0, 2.0))


def test_invalid_bounding_box_maps_to_python_error() -> None:
    points = [rs.Point(0.0, 0.0)]

    with pytest.raises(rs.InvalidBoundingBoxError):
        rs.hausdorff_clipped(points, points, rs.BoundingBox(10.0, -10.0, 0.0, 0.0))


def test_missing_densification_knobs_map_to_invalid_geometry_error() -> None:
    line = rs.LineString([(0.0, 0.0), (0.0, 1.0)])

    with pytest.raises(rs.InvalidGeometryError):
        line.densify(max_segment_length_m=None, max_segment_angle_deg=None)


def test_degenerate_polyline_maps_to_invalid_geometry_error() -> None:
    with pytest.raises(rs.InvalidGeometryError):
        rs.LineString([(0.0, 0.0), (0.0, 0.0)])


def test_unmapped_variant_falls_back_to_base_exception() -> None:
    with pytest.raises(rs.GeodistError):
        raise rs.GeodistError("synthetic geodist error")
