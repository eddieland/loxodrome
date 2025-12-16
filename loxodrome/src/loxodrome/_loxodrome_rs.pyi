"""Typing stub for the compiled loxodrome Rust extension.

Exports stay intentionally small while Rust-backed geometry wrappers are built.
Keep this stub in sync with `loxodrome-rs/src/python.rs`.
"""

from typing import Final, Literal

EARTH_RADIUS_METERS: Final[float]

class GeodistError(ValueError): ...
class InvalidLatitudeError(GeodistError): ...
class InvalidLongitudeError(GeodistError): ...
class InvalidAltitudeError(GeodistError): ...
class InvalidDistanceError(GeodistError): ...
class InvalidRadiusError(GeodistError): ...
class InvalidEllipsoidError(GeodistError): ...
class InvalidBoundingBoxError(GeodistError): ...
class EmptyPointSetError(GeodistError): ...
class InvalidGeometryError(GeodistError): ...

class Ellipsoid:
    semi_major_axis_m: float
    semi_minor_axis_m: float

    def __init__(self, semi_major_axis_m: float, semi_minor_axis_m: float) -> None: ...
    @staticmethod
    def wgs84() -> Ellipsoid: ...
    def to_tuple(self) -> tuple[float, float]: ...

class Point:
    lat: float
    lon: float

    def __init__(self, lat: float, lon: float) -> None: ...
    def to_tuple(self) -> tuple[float, float]: ...

class Point3D:
    lat: float
    lon: float
    altitude_m: float

    def __init__(self, lat: float, lon: float, altitude_m: float) -> None: ...
    def to_tuple(self) -> tuple[float, float, float]: ...

class Polygon:
    def __init__(self, exterior: list[tuple[float, float]], holes: list[list[tuple[float, float]]]): ...
    def to_tuple(self) -> tuple[list[tuple[float, float]], list[list[tuple[float, float]]]]: ...

class LineString:
    def __init__(self, vertices: list[tuple[float, float]]): ...
    def to_tuple(self) -> list[tuple[float, float]]: ...
    def densify(
        self,
        max_segment_length_m: float | None = ...,
        max_segment_angle_deg: float | None = ...,
        sample_cap: int = ...,
    ) -> list[Point]: ...
    def __len__(self) -> int: ...

class DensificationOptions:
    max_segment_length_m: float | None
    max_segment_angle_deg: float | None
    sample_cap: int

    def __init__(
        self,
        max_segment_length_m: float | None = ...,
        max_segment_angle_deg: float | None = ...,
        sample_cap: int = ...,
    ) -> None: ...
    def to_tuple(self) -> tuple[float | None, float | None, int]: ...

class GeodesicSolution:
    distance_m: float
    initial_bearing_deg: float
    final_bearing_deg: float

    def to_tuple(self) -> tuple[float, float, float]: ...

class HausdorffDirectedWitness:
    distance_m: float
    origin_index: int
    candidate_index: int

    def to_tuple(self) -> tuple[float, int, int]: ...

class HausdorffWitness:
    distance_m: float
    a_to_b: HausdorffDirectedWitness
    b_to_a: HausdorffDirectedWitness

    def to_tuple(
        self,
    ) -> tuple[
        float,
        tuple[float, int, int],
        tuple[float, int, int],
    ]: ...

class PolylineDirectedWitness:
    distance_m: float
    source_part: int
    source_index: int
    target_part: int
    target_index: int
    source_coord: Point
    target_coord: Point

class PolylineHausdorffWitness:
    distance_m: float
    a_to_b: PolylineDirectedWitness
    b_to_a: PolylineDirectedWitness

class ChamferDirectedResult:
    distance_m: float
    witness: PolylineDirectedWitness | None

class ChamferResult:
    distance_m: float
    a_to_b: ChamferDirectedResult
    b_to_a: ChamferDirectedResult

class BoundingBox:
    min_lat: float
    max_lat: float
    min_lon: float
    max_lon: float

    def __init__(
        self,
        min_lat: float,
        max_lat: float,
        min_lon: float,
        max_lon: float,
    ) -> None: ...
    def to_tuple(self) -> tuple[float, float, float, float]: ...

def geodesic_distance(p1: Point, p2: Point) -> float: ...
def geodesic_distance_on_ellipsoid(p1: Point, p2: Point, ellipsoid: Ellipsoid) -> float: ...
def geodesic_with_bearings(p1: Point, p2: Point) -> GeodesicSolution: ...
def geodesic_with_bearings_on_ellipsoid(p1: Point, p2: Point, ellipsoid: Ellipsoid) -> GeodesicSolution: ...
def geodesic_distance_3d(p1: Point3D, p2: Point3D) -> float: ...
def hausdorff_directed(a: list[Point], b: list[Point]) -> HausdorffDirectedWitness: ...
def hausdorff(a: list[Point], b: list[Point]) -> HausdorffWitness: ...
def hausdorff_directed_clipped(
    a: list[Point], b: list[Point], bounding_box: BoundingBox
) -> HausdorffDirectedWitness: ...
def hausdorff_clipped(a: list[Point], b: list[Point], bounding_box: BoundingBox) -> HausdorffWitness: ...
def hausdorff_directed_polyline(
    a: list[LineString],
    b: list[LineString],
    options: DensificationOptions | None = ...,
) -> PolylineDirectedWitness: ...
def hausdorff_polyline(
    a: list[LineString],
    b: list[LineString],
    options: DensificationOptions | None = ...,
) -> PolylineHausdorffWitness: ...
def chamfer_directed_polyline(
    a: list[LineString],
    b: list[LineString],
    reduction: Literal["mean", "sum", "max"] = ...,
    options: DensificationOptions | None = ...,
) -> ChamferDirectedResult: ...
def chamfer_polyline(
    a: list[LineString],
    b: list[LineString],
    reduction: Literal["mean", "sum", "max"] = ...,
    options: DensificationOptions | None = ...,
) -> ChamferResult: ...
def hausdorff_directed_3d(a: list[Point3D], b: list[Point3D]) -> HausdorffDirectedWitness: ...
def hausdorff_3d(a: list[Point3D], b: list[Point3D]) -> HausdorffWitness: ...
def hausdorff_directed_clipped_3d(
    a: list[Point3D], b: list[Point3D], bounding_box: BoundingBox
) -> HausdorffDirectedWitness: ...
def hausdorff_clipped_3d(a: list[Point3D], b: list[Point3D], bounding_box: BoundingBox) -> HausdorffWitness: ...
def hausdorff_directed_polyline_clipped(
    a: list[LineString],
    b: list[LineString],
    bounding_box: BoundingBox,
    options: DensificationOptions | None = ...,
) -> PolylineDirectedWitness: ...
def hausdorff_polyline_clipped(
    a: list[LineString],
    b: list[LineString],
    bounding_box: BoundingBox,
    options: DensificationOptions | None = ...,
) -> PolylineHausdorffWitness: ...
def chamfer_directed_polyline_clipped(
    a: list[LineString],
    b: list[LineString],
    bounding_box: BoundingBox,
    reduction: Literal["mean", "sum", "max"] = ...,
    options: DensificationOptions | None = ...,
) -> ChamferDirectedResult: ...
def chamfer_polyline_clipped(
    a: list[LineString],
    b: list[LineString],
    bounding_box: BoundingBox,
    reduction: Literal["mean", "sum", "max"] = ...,
    options: DensificationOptions | None = ...,
) -> ChamferResult: ...
def hausdorff_polygon_boundary(
    a: Polygon,
    b: Polygon,
    max_segment_length_m: float | None,
    max_segment_angle_deg: float | None,
    sample_cap: int,
) -> float: ...
def geodesic_distance_batch(
    origins_lat: object,
    origins_lon: object,
    destinations_lat: object,
    destinations_lon: object,
    ellipsoid: Ellipsoid | None = ...,
) -> list[float]: ...
def geodesic_with_bearings_batch(
    origins_lat: object,
    origins_lon: object,
    destinations_lat: object,
    destinations_lon: object,
    ellipsoid: Ellipsoid | None = ...,
) -> tuple[list[float], list[float], list[float]]: ...
def geodesic_distance_to_many(
    origin_lat: float,
    origin_lon: float,
    destinations_lat: object,
    destinations_lon: object,
    ellipsoid: Ellipsoid | None = ...,
) -> list[float]: ...
def polygon_area_batch(
    coords: object,
    ring_offsets: object,
    polygon_offsets: object,
    ellipsoid: Ellipsoid | None = ...,
) -> list[float]: ...

__all__ = [
    "EARTH_RADIUS_METERS",
    "GeodistError",
    "InvalidLatitudeError",
    "InvalidLongitudeError",
    "InvalidAltitudeError",
    "InvalidDistanceError",
    "InvalidRadiusError",
    "InvalidEllipsoidError",
    "InvalidBoundingBoxError",
    "EmptyPointSetError",
    "InvalidGeometryError",
    "Ellipsoid",
    "Point",
    "Point3D",
    "Polygon",
    "LineString",
    "DensificationOptions",
    "GeodesicSolution",
    "HausdorffDirectedWitness",
    "HausdorffWitness",
    "PolylineDirectedWitness",
    "PolylineHausdorffWitness",
    "ChamferDirectedResult",
    "ChamferResult",
    "BoundingBox",
    "geodesic_distance",
    "geodesic_distance_on_ellipsoid",
    "geodesic_distance_3d",
    "geodesic_with_bearings",
    "geodesic_with_bearings_on_ellipsoid",
    "hausdorff_directed",
    "hausdorff",
    "hausdorff_directed_clipped",
    "hausdorff_clipped",
    "hausdorff_directed_polyline",
    "hausdorff_polyline",
    "chamfer_directed_polyline",
    "chamfer_polyline",
    "hausdorff_directed_3d",
    "hausdorff_3d",
    "hausdorff_directed_clipped_3d",
    "hausdorff_clipped_3d",
    "hausdorff_directed_polyline_clipped",
    "hausdorff_polyline_clipped",
    "chamfer_directed_polyline_clipped",
    "chamfer_polyline_clipped",
    "hausdorff_polygon_boundary",
    "geodesic_distance_batch",
    "geodesic_with_bearings_batch",
    "geodesic_distance_to_many",
    "polygon_area_batch",
]

# Upcoming Rust-backed geometry handles will mirror the Rust structs once exposed:
# - Additional geometry containers will be added incrementally once the kernels are wired.
