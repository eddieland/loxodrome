from __future__ import annotations

import importlib.util

import pytest
from pytest import approx

if importlib.util.find_spec("geodist._geodist_rs") is None:
    pytest.skip("Rust extension is not built; skipping surface checks.", allow_module_level=True)

import geodist


def test_constant_reexported() -> None:
    assert geodist.EARTH_RADIUS_METERS == approx(6_371_008.8)


def test_public_api_reflects_trimmed_surface() -> None:
    assert geodist.__all__ == (
        "EARTH_RADIUS_METERS",
        "GeodistError",
        "InvalidGeometryError",
        "BoundingBox",
        "Point",
        "Point3D",
        "GeodesicResult",
        "geodesic_distance",
        "geodesic_distance_3d",
        "geodesic_with_bearings",
        "hausdorff",
        "hausdorff_3d",
        "hausdorff_clipped",
        "hausdorff_clipped_3d",
        "hausdorff_directed",
        "hausdorff_directed_3d",
        "hausdorff_directed_clipped",
        "hausdorff_directed_clipped_3d",
    )

    # Ensure the public Point wrapper is wired to the module import.
    assert geodist.Point.__name__ == "Point"
    assert geodist.Point3D.__name__ == "Point3D"
    assert geodist.BoundingBox.__name__ == "BoundingBox"
    assert geodist.GeodesicResult.__name__ == "GeodesicResult"
