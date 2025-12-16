from __future__ import annotations

import numpy as np
import pytest

from loxodrome import InvalidGeometryError, ops
from loxodrome import vectorized as vz
from loxodrome.geometry import Point


def test_points_from_coords_numpy_roundtrip() -> None:
    coords = np.array([[0.0, 0.0], [1.0, 1.0]], dtype=np.float64)
    batch = vz.points_from_coords(coords)
    np.testing.assert_allclose(batch.to_numpy(), coords)
    assert batch.to_python() == [(0.0, 0.0), (1.0, 1.0)]


def test_points_from_coords_reports_index() -> None:
    coords = np.array([[0.0, 0.0], [95.0, 0.0]], dtype=np.float64)
    with pytest.raises(InvalidGeometryError, match="index 1"):
        vz.points_from_coords(coords)


def test_points_from_coords_rejects_non_sequence_rows() -> None:
    with pytest.raises(InvalidGeometryError, match="coords must be at least 2-D"):
        vz.points_from_coords([1.0, 2.0])


def test_points3d_from_coords_rejects_mismatched_altitude() -> None:
    with pytest.raises(InvalidGeometryError, match="altitude_m must match coordinate length"):
        vz.points3d_from_coords([0.0, 1.0], [0.0, 1.0], [10.0])


def test_geodesic_distance_batch_matches_scalar() -> None:
    origins = vz.points_from_coords([(0.0, 0.0), (0.0, 0.0)])
    destinations = vz.points_from_coords([(0.0, 1.0), (1.0, 0.0)])

    distances = vz.geodesic_distance_batch(origins, destinations).to_numpy()
    expected = np.array(
        [
            ops.geodesic_distance(Point(0.0, 0.0), Point(0.0, 1.0)),
            ops.geodesic_distance(Point(0.0, 0.0), Point(1.0, 0.0)),
        ],
        dtype=np.float64,
    )

    np.testing.assert_allclose(distances, expected)


def test_geodesic_distance_to_many_reuses_origin() -> None:
    origin = Point(0.0, 0.0)
    destinations = vz.points_from_coords([(0.0, 1.0), (1.0, 0.0)])

    distances = vz.geodesic_distance_to_many(origin, destinations).to_numpy()
    assert distances.shape == (2,)
    assert distances[0] == pytest.approx(distances[1])


def test_area_batch_returns_expected_square() -> None:
    coords = np.array(
        [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
        ],
        dtype=np.float64,
    )
    ring_offsets = [0, len(coords)]
    polygon_offsets = [0, 1]
    polygons = vz.polygons_from_coords(coords, ring_offsets, polygon_offsets)

    areas = vz.area_batch(polygons).to_numpy()
    assert areas.shape == (1,)
    np.testing.assert_allclose(areas[0], 12_308_778_361.469452, rtol=1e-6)


def test_polylines_roundtrip_numpy_and_python() -> None:
    coords = np.array([[0.0, 0.0], [0.0, 1.0], [1.0, 1.0]], dtype=np.float64)
    offsets = np.array([0, len(coords)], dtype=np.int64)

    batch = vz.polylines_from_coords(coords, offsets)  # type: ignore[arg-type]
    coords_np, offsets_np = batch.to_numpy()

    np.testing.assert_allclose(coords_np, coords)
    np.testing.assert_array_equal(offsets_np, offsets)
    assert batch.to_python() == ([(0.0, 0.0), (0.0, 1.0), (1.0, 1.0)], [0, len(coords)])


def test_polylines_reject_nonzero_offset_start() -> None:
    coords = [(0.0, 0.0), (1.0, 1.0)]
    with pytest.raises(InvalidGeometryError, match="must start at 0"):
        vz.polylines_from_coords(coords, [1, 2])


def test_polygons_roundtrip_numpy_and_python() -> None:
    coords = np.array(
        [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
        ],
        dtype=np.float64,
    )
    ring_offsets = np.array([0, len(coords)], dtype=np.int64)
    polygon_offsets = np.array([0, 1], dtype=np.int64)

    batch = vz.polygons_from_coords(coords, ring_offsets, polygon_offsets)  # type: ignore[arg-type]
    coords_np, ring_offsets_np, polygon_offsets_np = batch.to_numpy()

    np.testing.assert_allclose(coords_np, coords)
    np.testing.assert_array_equal(ring_offsets_np, ring_offsets)
    np.testing.assert_array_equal(polygon_offsets_np, polygon_offsets)
    assert batch.to_python() == (
        [(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)],
        [0, len(coords)],
        [0, 1],
    )


def test_polygons_reject_non_monotonic_offsets() -> None:
    coords = [(0.0, 0.0), (1.0, 1.0)]
    with pytest.raises(InvalidGeometryError, match="monotonically increasing"):
        vz.polygons_from_coords(coords, [0, 2, 1], [0, 1])


def test_geodesic_with_bearings_batch_matches_scalar() -> None:
    origins = vz.points_from_coords([(0.0, 0.0), (0.0, 0.0)])
    destinations = vz.points_from_coords([(0.0, 1.0), (1.0, 0.0)])

    result = vz.geodesic_with_bearings_batch(origins, destinations)
    expected_a = ops.geodesic_with_bearings(Point(0.0, 0.0), Point(0.0, 1.0))
    expected_b = ops.geodesic_with_bearings(Point(0.0, 0.0), Point(1.0, 0.0))

    np.testing.assert_allclose(result.distance_m, [expected_a.distance_m, expected_b.distance_m])
    np.testing.assert_allclose(
        result.initial_bearing_deg, [expected_a.initial_bearing_deg, expected_b.initial_bearing_deg]
    )
    np.testing.assert_allclose(result.final_bearing_deg, [expected_a.final_bearing_deg, expected_b.final_bearing_deg])


def test_distance_result_list_path() -> None:
    batch = vz.points_from_coords([(0.0, 0.0), (0.0, 1.0)])
    result = vz.geodesic_distance_batch(batch, batch)
    assert isinstance(result.distance_m, np.ndarray)
    assert result.to_python() == pytest.approx(result.distance_m.tolist())
