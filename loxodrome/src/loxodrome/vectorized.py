"""Vectorized batch constructors and operations.

This module strictly requires numpy to be installed. Importing it will raise ImportError if numpy is missing.

At a high-level, the goal is to minimize the number of times we have to cross the Python/Rust FFI boundary by batching
operations on multiple geometries at once. The batch geometry classes here are thin wrappers around contiguous buffers
of coordinates and offsets, with conversion helpers to/from NumPy arrays and Python lists.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Sequence, SupportsFloat, TypeAlias, cast

import numpy as _np
import numpy.typing as _npt

from . import _loxodrome_rs
from .errors import InvalidGeometryError
from .geometry import Ellipsoid, Point, _coerce_point_like
from .types import Point as PointTuple
from .types import Point3D as Point3DTuple

FloatArray: TypeAlias = _npt.NDArray[_np.float64]
IntArray: TypeAlias = _npt.NDArray[_np.int64]

_LAT_MIN = -90.0
_LAT_MAX = 90.0
_LON_MIN = -180.0
_LON_MAX = 180.0

ArrayLike: TypeAlias = Sequence[SupportsFloat] | Sequence[Sequence[SupportsFloat]] | FloatArray
FloatBuffer: TypeAlias = FloatArray | list[float]
IntBuffer: TypeAlias = IntArray | list[int]
CoordMatrix: TypeAlias = FloatArray | list[tuple[float, float]]


def _validate_scalar(value: SupportsFloat, index: int, *, name: str, min_value: float, max_value: float) -> float:
    try:
        numeric_value = float(value)
    except (TypeError, ValueError) as exc:
        raise InvalidGeometryError(f"index {index}: {name} must be convertible to float") from exc

    if not math.isfinite(numeric_value):
        raise InvalidGeometryError(f"index {index}: {name} must be finite, got {numeric_value!r}")

    if numeric_value < min_value or numeric_value > max_value:
        raise InvalidGeometryError(f"index {index}: {name} {numeric_value} outside [{min_value}, {max_value}]")

    return numeric_value


def _validate_altitude(value: SupportsFloat, index: int) -> float:
    try:
        numeric_value = float(value)
    except (TypeError, ValueError) as exc:
        raise InvalidGeometryError(f"index {index}: altitude_m must be convertible to float") from exc

    if not math.isfinite(numeric_value):
        raise InvalidGeometryError(f"index {index}: altitude_m must be finite, got {numeric_value!r}")

    return numeric_value


def _validate_lat_lon_numpy(lat: FloatArray, lon: FloatArray, *, context: str) -> None:
    if lat.shape != lon.shape:
        raise InvalidGeometryError(f"{context}: lat and lon must share the same shape")

    if lat.ndim != 1:
        raise InvalidGeometryError(f"{context}: expected 1-D lat/lon arrays, got {lat.ndim}D")

    non_finite = ~_np.isfinite(lat) | ~_np.isfinite(lon)
    if non_finite.any():
        index = int(_np.argmax(non_finite))
        value = lat[index] if not _np.isfinite(lat[index]) else lon[index]
        axis = "lat" if not _np.isfinite(lat[index]) else "lon"
        raise InvalidGeometryError(f"index {index}: {axis} must be finite, got {float(value)!r}")

    lat_out_of_bounds = (lat < _LAT_MIN) | (lat > _LAT_MAX)
    if lat_out_of_bounds.any():
        index = int(_np.argmax(lat_out_of_bounds))
        raise InvalidGeometryError(f"index {index}: lat {float(lat[index])} outside [{_LAT_MIN}, {_LAT_MAX}]")

    lon_out_of_bounds = (lon < _LON_MIN) | (lon > _LON_MAX)
    if lon_out_of_bounds.any():
        index = int(_np.argmax(lon_out_of_bounds))
        raise InvalidGeometryError(f"index {index}: lon {float(lon[index])} outside [{_LON_MIN}, {_LON_MAX}]")


def _coerce_point_columns(lat_deg: ArrayLike, lon_deg: ArrayLike | None) -> tuple[FloatBuffer, FloatBuffer]:
    if isinstance(lat_deg, (_np.ndarray,)) or isinstance(lon_deg, (_np.ndarray,)) or lon_deg is None:
        if lon_deg is None:
            coords = _np.ascontiguousarray(_np.asarray(lat_deg, dtype=_np.float64))
            if coords.ndim < 2 or coords.shape[-1] < 2:
                raise InvalidGeometryError("coords must be at least 2-D with a trailing dimension of length 2")
            flat = coords.reshape(-1, coords.shape[-1])
            lat_array = _np.ascontiguousarray(flat[:, 0])
            lon_array = _np.ascontiguousarray(flat[:, 1])
        else:
            lat_array = cast(FloatArray, _np.ascontiguousarray(_np.asarray(lat_deg, dtype=_np.float64)).reshape(-1))
            lon_array = cast(FloatArray, _np.ascontiguousarray(_np.asarray(lon_deg, dtype=_np.float64)).reshape(-1))

        _validate_lat_lon_numpy(lat_array, lon_array, context="points_from_coords")

        arrays: tuple[FloatArray, FloatArray] = (lat_array, lon_array)
        return arrays

    lat_list: list[float] = []
    lon_list: list[float] = []

    lat_iterable: list[SupportsFloat] = list(cast(Sequence[SupportsFloat], lat_deg))
    lon_iterable: list[SupportsFloat] = list(cast(Sequence[SupportsFloat], lon_deg))
    if len(lat_iterable) != len(lon_iterable):
        raise InvalidGeometryError(f"lat and lon must share length, got {len(lat_iterable)} and {len(lon_iterable)}")

    for index, (lat_value, lon_value) in enumerate(zip(lat_iterable, lon_iterable)):
        lat_list.append(_validate_scalar(lat_value, index, name="lat", min_value=_LAT_MIN, max_value=_LAT_MAX))
        lon_list.append(_validate_scalar(lon_value, index, name="lon", min_value=_LON_MIN, max_value=_LON_MAX))

    return lat_list, lon_list


def _coerce_altitudes(altitude_m: Sequence[SupportsFloat] | FloatArray) -> FloatBuffer:
    if isinstance(altitude_m, (_np.ndarray,)):
        alt_array = cast(FloatArray, _np.ascontiguousarray(_np.asarray(altitude_m, dtype=_np.float64)).reshape(-1))
        non_finite = ~_np.isfinite(alt_array)
        if non_finite.any():
            index = int(_np.argmax(non_finite))
            raise InvalidGeometryError(f"index {index}: altitude_m must be finite")
        return alt_array

    altitudes: list[float] = []
    for index, value in enumerate(altitude_m):
        altitudes.append(_validate_altitude(value, index))
    return altitudes


def _validate_offsets(offsets: Sequence[int], *, name: str, expected_final: int) -> None:
    if not offsets:
        raise InvalidGeometryError(f"{name} must contain at least one entry")

    if offsets[0] != 0:
        raise InvalidGeometryError(f"{name} must start at 0, got {offsets[0]}")

    for previous, current in zip(offsets, offsets[1:]):
        if current < previous:
            raise InvalidGeometryError(f"{name} must be monotonically increasing")

    if offsets[-1] != expected_final:
        raise InvalidGeometryError(f"{name} must end at {expected_final}, got {offsets[-1]}")


def _coerce_offset_array(offsets: Sequence[int], *, name: str, expected_final: int) -> tuple[IntBuffer, int]:
    if isinstance(offsets, (_np.ndarray,)):
        offset_array = cast(IntArray, _np.ascontiguousarray(_np.asarray(offsets, dtype=_np.int64)).reshape(-1))
        offset_list = offset_array.tolist()
        _validate_offsets(offset_list, name=name, expected_final=expected_final)
        return offset_array, len(offset_list)

    offset_list = [int(value) for value in offsets]
    _validate_offsets(offset_list, name=name, expected_final=expected_final)
    return offset_list, len(offset_list)


def _coerce_coord_matrix(coords: ArrayLike, *, dims: int) -> tuple[int, CoordMatrix]:
    if isinstance(coords, (_np.ndarray,)):
        coord_array = _np.ascontiguousarray(_np.asarray(coords, dtype=_np.float64))
        if coord_array.ndim < 2 or coord_array.shape[-1] < dims:
            raise InvalidGeometryError(f"coords must be at least 2-D with trailing dimension {dims}")
        flat = coord_array.reshape(-1, coord_array.shape[-1])
        lat = _np.ascontiguousarray(flat[:, 0])
        lon = _np.ascontiguousarray(flat[:, 1])
        _validate_lat_lon_numpy(lat, lon, context="coords")
        return flat.shape[0], coord_array

    rows = list(coords)
    coord_rows: list[tuple[float, float]] = []
    for index, row in enumerate(rows):
        if not isinstance(row, Sequence) or len(row) < dims:
            raise InvalidGeometryError(f"index {index}: expected a sequence of length {dims}")
        lat_value = _validate_scalar(row[0], index, name="lat", min_value=_LAT_MIN, max_value=_LAT_MAX)
        lon_value = _validate_scalar(row[1], index, name="lon", min_value=_LON_MIN, max_value=_LON_MAX)
        coord_rows.append((lat_value, lon_value))
    return len(coord_rows), coord_rows


@dataclass(frozen=True, slots=True)
class PointBatch:
    """Batch of 2D geographic points expressed in degrees."""

    _lat: FloatBuffer = field(repr=False)
    _lon: FloatBuffer = field(repr=False)

    @property
    def lat_deg(self) -> FloatBuffer:
        """Latitude buffer in degrees."""
        return self._lat

    @property
    def lon_deg(self) -> FloatBuffer:
        """Longitude buffer in degrees."""
        return self._lon

    def to_numpy(self) -> FloatArray:
        """Return coordinates as a contiguous (N, 2) float64 array."""
        stacked = _np.stack((self._lat, self._lon), axis=-1)
        return _np.ascontiguousarray(stacked, dtype=_np.float64)

    def to_python(self) -> list[PointTuple]:
        """Return coordinates as a list of (lat, lon) tuples."""
        if isinstance(self._lat, _np.ndarray) and isinstance(self._lon, _np.ndarray):
            return list(map(tuple, _np.stack((self._lat, self._lon), axis=-1).tolist()))
        return list(zip(self._lat, self._lon))

    def __len__(self) -> int:
        """Number of points in the batch."""
        return len(self._lat)


@dataclass(frozen=True, slots=True)
class Point3DBatch:
    """Batch of 3D geographic points expressed in degrees + meters."""

    _lat: FloatBuffer = field(repr=False)
    _lon: FloatBuffer = field(repr=False)
    _alt: FloatBuffer = field(repr=False)

    @property
    def lat_deg(self) -> FloatBuffer:
        """Latitude buffer in degrees."""
        return self._lat

    @property
    def lon_deg(self) -> FloatBuffer:
        """Longitude buffer in degrees."""
        return self._lon

    @property
    def altitude_m(self) -> FloatBuffer:
        """Altitude buffer in meters."""
        return self._alt

    def to_numpy(self) -> FloatArray:
        """Return coordinates as a contiguous (N, 3) float64 array."""
        stacked = _np.stack((self._lat, self._lon, self._alt), axis=-1)
        return _np.ascontiguousarray(stacked, dtype=_np.float64)

    def to_python(self) -> list[Point3DTuple]:
        """Return coordinates as a list of (lat, lon, altitude_m) tuples."""
        if isinstance(self._lat, _np.ndarray):
            return list(map(tuple, _np.stack((self._lat, self._lon, self._alt), axis=-1).tolist()))
        return list(zip(self._lat, self._lon, self._alt))

    def __len__(self) -> int:
        """Number of points in the batch."""
        return len(self._lat)


@dataclass(frozen=True, slots=True)
class PolylineBatch:
    """Batch of polylines encoded via flat coordinates and part offsets."""

    coords: CoordMatrix
    offsets: IntBuffer

    def to_numpy(self) -> tuple[FloatArray, IntArray]:
        """Return contiguous coordinate and offset arrays."""
        coords = _np.ascontiguousarray(self.coords, dtype=_np.float64)
        offsets = _np.ascontiguousarray(self.offsets, dtype=_np.int64)
        return coords, offsets

    def to_python(self) -> tuple[list[PointTuple], list[int]]:
        """Return Python-native coordinates and offsets."""
        coord_list = (
            list(map(tuple, self.coords.tolist()))
            if isinstance(self.coords, _np.ndarray)
            else [tuple(row) for row in self.coords]
        )
        offset_list = self.offsets.tolist() if isinstance(self.offsets, _np.ndarray) else list(self.offsets)
        return coord_list, offset_list


@dataclass(frozen=True, slots=True)
class PolygonBatch:
    """Batch of polygons encoded with ring and polygon offsets."""

    coords: CoordMatrix
    ring_offsets: IntBuffer
    polygon_offsets: IntBuffer

    def to_numpy(self) -> tuple[FloatArray, IntArray, IntArray]:
        """Return contiguous coordinate, ring offset, and polygon offset arrays."""
        coords = _np.ascontiguousarray(self.coords, dtype=_np.float64)
        ring_offsets = _np.ascontiguousarray(self.ring_offsets, dtype=_np.int64)
        polygon_offsets = _np.ascontiguousarray(self.polygon_offsets, dtype=_np.int64)
        return coords, ring_offsets, polygon_offsets

    def to_python(self) -> tuple[list[tuple[float, float]], list[int], list[int]]:
        """Return Python-native coordinates and offsets."""
        coord_list = (
            list(map(tuple, self.coords.tolist()))
            if isinstance(self.coords, _np.ndarray)
            else [tuple(row) for row in self.coords]
        )
        ring_offsets = (
            self.ring_offsets.tolist() if isinstance(self.ring_offsets, _np.ndarray) else list(self.ring_offsets)
        )
        polygon_offsets = (
            self.polygon_offsets.tolist()
            if isinstance(self.polygon_offsets, _np.ndarray)
            else list(self.polygon_offsets)
        )
        return coord_list, ring_offsets, polygon_offsets


@dataclass(frozen=True, slots=True)
class DistanceResult:
    """Container for batch distances."""

    distance_m: FloatBuffer

    def to_numpy(self) -> FloatArray:
        """Return distances as a float64 NumPy array."""
        return _np.ascontiguousarray(self.distance_m, dtype=_np.float64)

    def to_python(self) -> list[float]:
        """Return distances as Python floats."""
        if isinstance(self.distance_m, _np.ndarray):
            return [float(value) for value in self.distance_m.tolist()]
        return list(self.distance_m)


@dataclass(frozen=True, slots=True)
class BearingsResult:
    """Container for batch distances and bearings."""

    distance_m: FloatBuffer
    initial_bearing_deg: FloatBuffer
    final_bearing_deg: FloatBuffer

    def to_numpy(self) -> tuple[FloatArray, FloatArray, FloatArray]:
        """Return distances and bearings as float64 NumPy arrays."""
        return (
            _np.ascontiguousarray(self.distance_m, dtype=_np.float64),
            _np.ascontiguousarray(self.initial_bearing_deg, dtype=_np.float64),
            _np.ascontiguousarray(self.final_bearing_deg, dtype=_np.float64),
        )

    def to_python(self) -> tuple[list[float], list[float], list[float]]:
        """Return distances and bearings as Python floats."""
        if (
            isinstance(self.distance_m, _np.ndarray)
            and isinstance(self.initial_bearing_deg, _np.ndarray)
            and isinstance(self.final_bearing_deg, _np.ndarray)
        ):
            return (
                [float(value) for value in self.distance_m.tolist()],
                [float(value) for value in self.initial_bearing_deg.tolist()],
                [float(value) for value in self.final_bearing_deg.tolist()],
            )

        return (
            list(self.distance_m),
            list(self.initial_bearing_deg),
            list(self.final_bearing_deg),
        )


@dataclass(frozen=True, slots=True)
class AreaResult:
    """Container for polygon areas."""

    area_m2: FloatBuffer

    def to_numpy(self) -> FloatArray:
        """Return areas as a float64 NumPy array."""
        return _np.ascontiguousarray(self.area_m2, dtype=_np.float64)

    def to_python(self) -> list[float]:
        """Return areas as Python floats."""
        if isinstance(self.area_m2, _np.ndarray):
            return [float(value) for value in self.area_m2.tolist()]
        return list(self.area_m2)


def points_from_coords(lat_deg: ArrayLike, lon_deg: ArrayLike | None = None) -> PointBatch:
    """Construct a PointBatch from latitude/longitude buffers."""
    lat, lon = _coerce_point_columns(lat_deg, lon_deg)
    return PointBatch(lat, lon)


def points3d_from_coords(
    lat_deg: ArrayLike,
    lon_deg: ArrayLike,
    altitude_m: Sequence[SupportsFloat] | FloatArray,
) -> Point3DBatch:
    """Construct a Point3DBatch from latitude/longitude/altitude buffers."""
    lat, lon = _coerce_point_columns(lat_deg, lon_deg)
    alt = _coerce_altitudes(altitude_m)
    if len(lat) != len(alt):
        raise InvalidGeometryError(f"altitude_m must match coordinate length, got {len(alt)} vs {len(lat)}")
    return Point3DBatch(lat, lon, alt)


def polylines_from_coords(coords: ArrayLike, offsets: Sequence[int]) -> PolylineBatch:
    """Construct a PolylineBatch from flat coordinates and offsets."""
    length, coerced_coords = _coerce_coord_matrix(coords, dims=2)
    coerced_offsets, _ = _coerce_offset_array(offsets, name="offsets", expected_final=length)
    return PolylineBatch(coerced_coords, coerced_offsets)


def polygons_from_coords(
    coords: ArrayLike,
    ring_offsets: Sequence[int],
    polygon_offsets: Sequence[int],
) -> PolygonBatch:
    """Construct a PolygonBatch mirroring Arrow geometry layout."""
    length, coerced_coords = _coerce_coord_matrix(coords, dims=2)
    coerced_ring_offsets, ring_offset_length = _coerce_offset_array(
        ring_offsets, name="ring_offsets", expected_final=length
    )
    coerced_polygon_offsets, _ = _coerce_offset_array(
        polygon_offsets, name="polygon_offsets", expected_final=ring_offset_length - 1
    )
    return PolygonBatch(coerced_coords, coerced_ring_offsets, coerced_polygon_offsets)


def _coerce_point_batch(value: PointBatch | ArrayLike) -> PointBatch:
    if isinstance(value, PointBatch):
        return value

    return points_from_coords(value, None)


def _coerce_ellipsoid(ellipsoid: Ellipsoid | Sequence[float] | None) -> Ellipsoid | None:
    if ellipsoid is None:
        return None

    if isinstance(ellipsoid, Ellipsoid):
        return ellipsoid

    return Ellipsoid(float(ellipsoid[0]), float(ellipsoid[1]))


def geodesic_distance_batch(
    origins: PointBatch | ArrayLike,
    destinations: PointBatch | ArrayLike,
    *,
    ellipsoid: Ellipsoid | Sequence[float] | None = None,
) -> DistanceResult:
    """Compute pairwise great-circle distances between origin and destination batches."""
    origin_batch = _coerce_point_batch(origins)
    destination_batch = _coerce_point_batch(destinations)

    if len(origin_batch) != len(destination_batch):
        raise InvalidGeometryError(
            f"origins and destinations must share length, got {len(origin_batch)} and {len(destination_batch)}"
        )

    model = _coerce_ellipsoid(ellipsoid)
    distances = _loxodrome_rs.geodesic_distance_batch(
        origin_batch.lat_deg,
        origin_batch.lon_deg,
        destination_batch.lat_deg,
        destination_batch.lon_deg,
        ellipsoid=model._handle if model else None,
    )

    return DistanceResult(_np.asarray(distances, dtype=_np.float64))


def geodesic_with_bearings_batch(
    origins: PointBatch | ArrayLike,
    destinations: PointBatch | ArrayLike,
    *,
    ellipsoid: Ellipsoid | Sequence[float] | None = None,
) -> BearingsResult:
    """Compute distances and bearings for paired origin/destination points."""
    origin_batch = _coerce_point_batch(origins)
    destination_batch = _coerce_point_batch(destinations)
    if len(origin_batch) != len(destination_batch):
        raise InvalidGeometryError(
            f"origins and destinations must share length, got {len(origin_batch)} and {len(destination_batch)}"
        )

    model = _coerce_ellipsoid(ellipsoid)
    distances, initial_bearings, final_bearings = _loxodrome_rs.geodesic_with_bearings_batch(
        origin_batch.lat_deg,
        origin_batch.lon_deg,
        destination_batch.lat_deg,
        destination_batch.lon_deg,
        ellipsoid=model._handle if model else None,
    )

    return BearingsResult(
        _np.asarray(distances, dtype=_np.float64),
        _np.asarray(initial_bearings, dtype=_np.float64),
        _np.asarray(final_bearings, dtype=_np.float64),
    )


def geodesic_distance_to_many(
    origin: Point | PointTuple,
    destinations: PointBatch | ArrayLike,
    *,
    ellipsoid: Ellipsoid | Sequence[float] | None = None,
) -> DistanceResult:
    """Compute distances from a single origin to many destination points."""
    lat, lon = _coerce_point_like(origin)
    destination_batch = _coerce_point_batch(destinations)
    model = _coerce_ellipsoid(ellipsoid)

    distances = _loxodrome_rs.geodesic_distance_to_many(
        lat,
        lon,
        destination_batch.lat_deg,
        destination_batch.lon_deg,
        ellipsoid=model._handle if model else None,
    )

    return DistanceResult(_np.asarray(distances, dtype=_np.float64))


def area_batch(
    polygons: PolygonBatch,
    *,
    ellipsoid: Ellipsoid | Sequence[float] | None = None,
) -> AreaResult:
    """Compute polygon areas for a batch of polygons."""
    model = _coerce_ellipsoid(ellipsoid)
    areas = _loxodrome_rs.polygon_area_batch(
        polygons.coords,
        polygons.ring_offsets,
        polygons.polygon_offsets,
        ellipsoid=model._handle if model else None,
    )

    return AreaResult(_np.asarray(areas, dtype=_np.float64))


__all__ = [
    "PointBatch",
    "Point3DBatch",
    "PolylineBatch",
    "PolygonBatch",
    "DistanceResult",
    "BearingsResult",
    "AreaResult",
    "points_from_coords",
    "points3d_from_coords",
    "polylines_from_coords",
    "polygons_from_coords",
    "geodesic_distance_batch",
    "geodesic_with_bearings_batch",
    "geodesic_distance_to_many",
    "area_batch",
]
