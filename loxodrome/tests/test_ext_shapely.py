from __future__ import annotations

import pytest

from loxodrome import BoundingBox, InvalidGeometryError, LineString, Point, Point3D
from loxodrome.ext.shapely import from_shapely, to_shapely


def test_roundtrip_converts_between_point_types() -> None:
    shapely_point = pytest.importorskip("shapely.geometry").Point

    source_point = Point(12.5, -45.0)
    converted = to_shapely(source_point)
    assert isinstance(converted, shapely_point)
    assert converted.x == pytest.approx(-45.0)
    assert converted.y == pytest.approx(12.5)

    restored = from_shapely(converted)
    assert restored == source_point


def test_roundtrip_converts_3d_points() -> None:
    shapely_point = pytest.importorskip("shapely.geometry").Point

    source_point = Point3D(1.0, 2.0, 3.0)
    converted = to_shapely(source_point)
    assert isinstance(converted, shapely_point)
    assert converted.has_z is True
    assert converted.x == pytest.approx(2.0)
    assert converted.y == pytest.approx(1.0)
    assert converted.z == pytest.approx(3.0)

    restored = from_shapely(converted)
    assert isinstance(restored, Point3D)
    assert restored == source_point


def test_from_shapely_rejects_non_points() -> None:
    shapely_multipoint = pytest.importorskip("shapely.geometry").MultiPoint

    with pytest.raises(TypeError):
        from_shapely(shapely_multipoint([(0.0, 0.0), (1.0, 0.0)]))


def test_roundtrip_converts_bounding_boxes() -> None:
    shapely_box = pytest.importorskip("shapely.geometry").box
    shapely_polygon = pytest.importorskip("shapely.geometry").Polygon

    bbox = BoundingBox(10.0, 20.0, -5.0, 15.0)
    converted = to_shapely(bbox)
    assert isinstance(converted, shapely_polygon)
    assert converted.bounds == pytest.approx((-5.0, 10.0, 15.0, 20.0))

    restored = from_shapely(converted)
    assert restored == bbox

    # Ensure rectangles remain supported regardless of construction helper.
    restored_from_box = from_shapely(shapely_box(-5.0, 10.0, 15.0, 20.0))
    assert restored_from_box == bbox


def test_from_shapely_rejects_non_rectangular_polygons() -> None:
    shapely_polygon = pytest.importorskip("shapely.geometry").Polygon

    triangle = shapely_polygon([(0.0, 0.0), (2.0, 0.0), (1.0, 3.0)])
    with pytest.raises(InvalidGeometryError):
        from_shapely(triangle)


def test_linestring_roundtrip() -> None:
    shapely_linestring = pytest.importorskip("shapely.geometry").LineString

    line = LineString([(0.0, 0.0), (0.0, 1.0)])
    converted = to_shapely(line)
    assert isinstance(converted, shapely_linestring)
    coords = list(converted.coords)
    assert coords[0] == pytest.approx((0.0, 0.0))  # lon, lat ordering
    assert coords[1] == pytest.approx((1.0, 0.0))

    restored = from_shapely(converted)
    assert isinstance(restored, LineString)
    assert restored.to_tuple() == line.to_tuple()


def test_from_shapely_rejects_3d_linestring() -> None:
    shapely_linestring = pytest.importorskip("shapely.geometry").LineString

    with pytest.raises(InvalidGeometryError):
        from_shapely(shapely_linestring([(0.0, 0.0, 1.0), (1.0, 1.0, 2.0)]))


def test_to_shapely_rejects_unknown_geometries() -> None:
    class Dummy: ...

    with pytest.raises(TypeError):
        to_shapely(Dummy())  # type: ignore[arg-type]
