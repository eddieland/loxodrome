"""IO helpers for serializing and parsing geometry representations."""

from __future__ import annotations

from typing import Any, Mapping, Protocol, runtime_checkable

from . import _geodist_rs
from .errors import (
    CRSValidationError,
    GeometryTypeError,
    InvalidGeometryError,
    KernelExecutionError,
    KernelUnavailableError,
)
from .geometry import Geometry
from .types import CRSLike, GeoJSONLike, GeometryHandle


@runtime_checkable
class _KernelCallable(Protocol):
    def __call__(self, *args: Any, **kwargs: Any) -> Any: ...


def _validate_crs(crs: CRSLike) -> CRSLike:
    if crs is None or isinstance(crs, (int, str)):
        return crs
    raise CRSValidationError("CRS must be provided as an EPSG integer or authority string.")


def _require_kernel(function_name: str) -> _KernelCallable:
    maybe_kernel = getattr(_geodist_rs, function_name, None)
    if not isinstance(maybe_kernel, _KernelCallable):
        raise KernelUnavailableError(
            f"Kernel function `{function_name}` is unavailable; rebuild the extension to enable this operation."
        )
    return maybe_kernel


def _require_geometry(geometry: Geometry, action: str) -> GeometryHandle:
    if not isinstance(geometry, Geometry):
        raise GeometryTypeError(f"{action} expects geometry inputs; received {type(geometry).__name__}.")
    return geometry._require_handle(action)


def _call_kernel(kernel: _KernelCallable, *args: Any, action: str, **kwargs: Any) -> Any:
    try:
        return kernel(*args, **kwargs)
    except KernelUnavailableError:
        raise
    except ValueError as exc:
        raise InvalidGeometryError(f"{action} failed validation: {exc}") from exc
    except TypeError as exc:
        raise InvalidGeometryError(f"{action} received unsupported input types: {exc}") from exc
    except Exception as exc:  # pragma: no cover - defensive guard for unexpected errors
        raise KernelExecutionError(f"{action} failed unexpectedly: {exc}") from exc


def _normalize_wkb(wkb: bytes | bytearray | memoryview, *, action: str) -> bytes:
    try:
        return bytes(wkb)
    except Exception as exc:  # pragma: no cover - guarded to surface clear errors
        raise InvalidGeometryError(f"{action} requires a bytes-like object.") from exc


def _validate_geojson(mapping: Mapping[str, object]) -> Mapping[str, object]:
    if not isinstance(mapping, Mapping):
        raise InvalidGeometryError("GeoJSON input must be provided as a mapping.")
    return mapping


def _resolve_crs(crs: CRSLike, *, fallback: object | None = None) -> CRSLike:
    if crs is not None:
        return _validate_crs(crs)
    if fallback is not None and isinstance(fallback, (int, str)):
        return _validate_crs(fallback)
    return _validate_crs(crs)


def loads_wkt(wkt: str, *, crs: CRSLike = None) -> Geometry:
    """Parse a WKT string into a geometry, preserving optional CRS metadata."""
    if not isinstance(wkt, str):
        raise InvalidGeometryError("WKT input must be a string.")
    crs = _validate_crs(crs)
    kernel = _require_kernel("loads_wkt")
    handle = _call_kernel(kernel, wkt, action="WKT parsing", crs=crs)
    return Geometry._from_handle(handle, crs=crs)


def dumps_wkt(geometry: Geometry) -> str:
    """Serialize a geometry to its WKT representation."""
    handle = _require_geometry(geometry, "WKT serialization")
    kernel = _require_kernel("dumps_wkt")
    wkt = _call_kernel(kernel, handle, action="WKT serialization", crs=geometry.crs)
    if not isinstance(wkt, str):
        raise KernelExecutionError("WKT serialization must return a string.")
    return wkt


def loads_wkb(wkb: bytes | bytearray | memoryview, *, crs: CRSLike = None) -> Geometry:
    """Parse WKB bytes into a geometry."""
    normalized = _normalize_wkb(wkb, action="WKB parsing")
    crs = _validate_crs(crs)
    kernel = _require_kernel("loads_wkb")
    handle = _call_kernel(kernel, normalized, action="WKB parsing", crs=crs)
    return Geometry._from_handle(handle, crs=crs)


def dumps_wkb(geometry: Geometry) -> bytes:
    """Serialize a geometry to WKB bytes."""
    handle = _require_geometry(geometry, "WKB serialization")
    kernel = _require_kernel("dumps_wkb")
    wkb = _call_kernel(kernel, handle, action="WKB serialization", crs=geometry.crs)
    if not isinstance(wkb, (bytes, bytearray, memoryview)):
        raise KernelExecutionError("WKB serialization must return a bytes-like object.")
    return bytes(wkb)


def loads_geojson(mapping: GeoJSONLike, *, crs: CRSLike = None) -> Geometry:
    """Parse a GeoJSON mapping into the corresponding geometry."""
    mapping = _validate_geojson(mapping)
    crs = _resolve_crs(crs, fallback=mapping.get("crs"))
    kernel = _require_kernel("loads_geojson")
    handle = _call_kernel(kernel, mapping, action="GeoJSON parsing", crs=crs)
    return Geometry._from_handle(handle, crs=crs)


def dumps_geojson(geometry: Geometry) -> GeoJSONLike:
    """Serialize a geometry to a GeoJSON mapping."""
    handle = _require_geometry(geometry, "GeoJSON serialization")
    kernel = _require_kernel("dumps_geojson")
    mapping = _call_kernel(kernel, handle, action="GeoJSON serialization", crs=geometry.crs)
    validated_mapping = _validate_geojson(mapping)
    return validated_mapping


__all__ = (
    "dumps_geojson",
    "dumps_wkb",
    "dumps_wkt",
    "loads_geojson",
    "loads_wkb",
    "loads_wkt",
)
