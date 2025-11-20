from __future__ import annotations

import importlib.util

import pytest

if importlib.util.find_spec("geodist._geodist_rs") is None:
    pytest.skip("Rust extension is not built; skipping Shapely interop checks.", allow_module_level=True)

from geodist import InvalidGeometryError, Point
from geodist.ext.shapely import from_shapely, to_shapely

SHAPELY_AVAILABLE = importlib.util.find_spec("shapely.geometry") is not None


@pytest.mark.skipif(SHAPELY_AVAILABLE, reason="Dependency is present; this guard test expects it missing.")
def test_to_shapely_requires_optional_dependency() -> None:
    with pytest.raises(ImportError):
        to_shapely(Point(0.0, 0.0))


@pytest.mark.skipif(not SHAPELY_AVAILABLE, reason="Shapely is not installed.")
def test_roundtrip_converts_between_point_types() -> None:
    shapely_point = pytest.importorskip("shapely.geometry").Point

    source_point = Point(12.5, -45.0)
    converted = to_shapely(source_point)
    assert isinstance(converted, shapely_point)
    assert converted.x == pytest.approx(-45.0)
    assert converted.y == pytest.approx(12.5)

    restored = from_shapely(converted)
    assert restored == source_point


@pytest.mark.skipif(not SHAPELY_AVAILABLE, reason="Shapely is not installed.")
def test_from_shapely_rejects_3d_points() -> None:
    shapely_point = pytest.importorskip("shapely.geometry").Point

    with pytest.raises(InvalidGeometryError):
        from_shapely(shapely_point(1.0, 2.0, 3.0))


@pytest.mark.skipif(not SHAPELY_AVAILABLE, reason="Shapely is not installed.")
def test_from_shapely_rejects_non_points() -> None:
    shapely_polygon = pytest.importorskip("shapely.geometry").Polygon

    with pytest.raises(TypeError):
        from_shapely(shapely_polygon([(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]))
