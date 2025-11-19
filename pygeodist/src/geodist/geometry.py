"""Geometry wrappers exposed by the geodist Python package."""

from __future__ import annotations

from typing import TYPE_CHECKING, NoReturn

from .errors import KernelUnavailableError
from .types import Coordinate, CoordinateSequence, CRSLike, GeometryHandle

if TYPE_CHECKING:
    from typing_extensions import Self


class Geometry:
    """Base class for immutable geometry wrappers backed by Rust handles.

    Geometry objects own their Rust handles; the handle stays alive as long as
    the Python object is referenced. All wrappers remain immutable to mirror
    Shapely/pygeos semantics and keep thread-safety guarantees simple.
    """

    __slots__ = ("_handle", "_crs", "_size_hint")

    def __init__(
        self,
        handle: GeometryHandle | None,
        *,
        crs: CRSLike = None,
        size_hint: int | None = None,
    ) -> None:
        """Initialize a geometry from an optional Rust handle, CRS, and size hint."""
        self._handle = handle
        self._crs = crs
        self._size_hint = size_hint

    @classmethod
    def _from_handle(
        cls,
        handle: GeometryHandle,
        *,
        crs: CRSLike = None,
        size_hint: int | None = None,
    ) -> Self:
        """Construct a Python wrapper around an opaque Rust handle."""
        instance = cls.__new__(cls)
        Geometry.__init__(instance, handle=handle, crs=crs, size_hint=size_hint)
        return instance

    @property
    def crs(self) -> CRSLike:
        """Declared coordinate reference system, if provided."""
        return self._crs

    def __eq__(self, other: object) -> bool:
        """Check equality between two geometry objects."""
        if not isinstance(other, Geometry):
            return NotImplemented
        if self.__class__ is not other.__class__:
            return False
        self_handle = self._require_handle("geometry comparison")
        other_handle = other._require_handle("geometry comparison")
        comparison = self_handle == other_handle
        if comparison is NotImplemented:
            return NotImplemented
        return bool(comparison)

    def __len__(self) -> int:
        """Return the number of vertices, rings, or geometries, depending on type."""
        if self._size_hint is not None:
            return self._size_hint
        handle = self._require_handle("length lookup")
        try:
            return len(handle)
        except TypeError as exc:
            raise KernelUnavailableError(
                "Geometry handles must expose __len__ so Python wrappers can reflect vertex counts."
            ) from exc

    def __repr__(self) -> str:
        """Return a debug representation of the geometry object."""
        handle_state = "uninitialized" if self._handle is None else "handle=<opaque>"
        crs_repr = f", crs={self._crs!r}" if self._crs is not None else ""
        size_repr = f", size_hint={self._size_hint}" if self._size_hint is not None else ""
        return f"{self.__class__.__name__}({handle_state}{crs_repr}{size_repr})"

    def _require_handle(self, action: str) -> GeometryHandle:
        if self._handle is None:
            self._raise_missing_kernel(action)
        return self._handle

    @staticmethod
    def _raise_missing_kernel(action: str) -> NoReturn:
        """Raise an error indicating that Rust kernels are not available."""
        raise KernelUnavailableError(
            f"Rust kernels are not available; cannot perform {action}. Build the extension to enable this operation."
        )


class Point(Geometry):
    """Immutable point geometry using an opaque Rust handle."""

    __slots__ = ()

    def __init__(self, coordinate: Coordinate, crs: CRSLike = None) -> None:
        """Initialize a point from a coordinate pair."""
        super().__init__(handle=None, crs=crs, size_hint=1)
        self._raise_missing_kernel("Point construction")

    @classmethod
    def from_xy(cls, x: float, y: float, crs: CRSLike = None) -> Point:
        """Build a point from an x/y pair."""
        return cls((x, y), crs=crs)


class LineString(Geometry):
    """Ordered sequence of coordinates forming a line."""

    __slots__ = ()

    def __init__(self, coordinates: CoordinateSequence, crs: CRSLike = None) -> None:
        """Initialize a linestring from a sequence of coordinates."""
        super().__init__(handle=None, crs=crs, size_hint=len(coordinates))
        self._raise_missing_kernel("LineString construction")


class Polygon(Geometry):
    """Polygon with an outer ring and optional interior holes."""

    __slots__ = ()

    def __init__(
        self,
        exterior: CoordinateSequence,
        holes: list[CoordinateSequence] | None = None,
        crs: CRSLike = None,
    ) -> None:
        """Initialize a polygon from an exterior ring and optional interior holes."""
        ring_count = 1 + len(holes or [])
        super().__init__(handle=None, crs=crs, size_hint=ring_count)
        self._raise_missing_kernel("Polygon construction")


class GeometryCollection(Geometry):
    """Heterogeneous collection of geometry objects."""

    __slots__ = ()

    def __init__(self, geometries: list[Geometry], crs: CRSLike = None) -> None:
        """Initialize a geometry collection from a list of geometries."""
        super().__init__(handle=None, crs=crs, size_hint=len(geometries))
        self._raise_missing_kernel("GeometryCollection construction")


__all__ = (
    "Geometry",
    "GeometryCollection",
    "LineString",
    "Point",
    "Polygon",
)
