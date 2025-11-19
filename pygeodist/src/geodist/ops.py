"""Geometry predicates and measures dispatched to the Rust kernels."""

from __future__ import annotations

from typing import Any, Protocol, runtime_checkable

from . import _geodist_rs
from .errors import (
    CRSValidationError,
    GeometryTypeError,
    InvalidGeometryError,
    KernelExecutionError,
    KernelUnavailableError,
)
from .geometry import Geometry, Point
from .types import CRSLike, GeometryHandle


@runtime_checkable
class _KernelCallable(Protocol):
    def __call__(self, *args: Any, **kwargs: Any) -> Any: ...


def _validate_crs(crs: CRSLike) -> CRSLike:
    if crs is None or isinstance(crs, (int, str)):
        return crs
    raise CRSValidationError("CRS must be provided as an EPSG integer or authority string.")


def _require_handle(geometry: Geometry, action: str) -> GeometryHandle:
    if not isinstance(geometry, Geometry):
        raise GeometryTypeError(f"{action} expects geometry inputs; received {type(geometry).__name__}.")
    return geometry._require_handle(action)


def _require_kernel(function_name: str) -> _KernelCallable:
    maybe_kernel = getattr(_geodist_rs, function_name, None)
    if not isinstance(maybe_kernel, _KernelCallable):
        raise KernelUnavailableError(
            f"Kernel function `{function_name}` is unavailable; rebuild the extension to enable this operation."
        )
    return maybe_kernel


def _call_kernel(kernel: _KernelCallable, *args: Any, action: str, **kwargs: Any) -> Any:
    try:
        return kernel(*args, **kwargs)
    except KernelUnavailableError:
        raise
    except ValueError as exc:
        raise InvalidGeometryError(f"{action} failed validation: {exc}") from exc
    except Exception as exc:  # pragma: no cover - safeguards unexpected failures
        raise KernelExecutionError(f"{action} failed unexpectedly: {exc}") from exc


def distance(a: Geometry, b: Geometry, *, crs: CRSLike = None) -> float:
    """Compute the geodesic distance between two geometries in meters."""
    crs = _validate_crs(crs)
    left = _require_handle(a, "distance computation")
    right = _require_handle(b, "distance computation")
    kernel = _require_kernel("distance")
    result = _call_kernel(kernel, left, right, action="distance computation", crs=crs)
    return float(result)


def equals(a: Geometry, b: Geometry) -> bool:
    """Return True when the geometries are topologically equal."""
    left = _require_handle(a, "equality testing")
    right = _require_handle(b, "equality testing")
    kernel = _require_kernel("equals")
    result = _call_kernel(kernel, left, right, action="equality testing")
    return bool(result)


def intersects(a: Geometry, b: Geometry) -> bool:
    """Return True when geometries have any boundary or interior intersection."""
    left = _require_handle(a, "intersection testing")
    right = _require_handle(b, "intersection testing")
    kernel = _require_kernel("intersects")
    result = _call_kernel(kernel, left, right, action="intersection testing")
    return bool(result)


def within(a: Geometry, b: Geometry) -> bool:
    """Return True when geometry `a` lies within geometry `b`."""
    left = _require_handle(a, "containment testing")
    right = _require_handle(b, "containment testing")
    kernel = _require_kernel("within")
    result = _call_kernel(kernel, left, right, action="containment testing")
    return bool(result)


def buffer(geometry: Geometry, distance_meters: float) -> Geometry:
    """Return a buffered geometry generated around the input geometry."""
    try:
        if distance_meters < 0:
            raise InvalidGeometryError("Buffer distance must be non-negative.")
    except TypeError as exc:
        raise InvalidGeometryError("Buffer distance must be a real number.") from exc
    handle = _require_handle(geometry, "buffering")
    kernel = _require_kernel("buffer")
    buffered_handle = _call_kernel(kernel, handle, distance_meters, action="buffering")
    return Geometry._from_handle(buffered_handle, crs=geometry.crs)


def centroid(geometry: Geometry) -> Point:
    """Return the centroid of the geometry."""
    handle = _require_handle(geometry, "centroid computation")
    kernel = _require_kernel("centroid")
    centroid_handle = _call_kernel(kernel, handle, action="centroid computation")
    return Point._from_handle(centroid_handle, crs=geometry.crs)


__all__ = (
    "buffer",
    "centroid",
    "distance",
    "equals",
    "intersects",
    "within",
)
