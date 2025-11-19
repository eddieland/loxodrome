"""Vectorized helpers for batch geometry operations."""

from __future__ import annotations

from collections.abc import Callable, Sequence
from typing import TypeVar

from . import ops
from .errors import GeodistError, VectorizationError
from .geometry import Geometry
from .types import CRSLike

try:  # pragma: no cover - optional dependency path
    import numpy as _np  # type: ignore[import-not-found]
except Exception:  # pragma: no cover - import guard
    _np = None

_T = TypeVar("_T")


def _ensure_lengths_match(left: Sequence[Geometry], right: Sequence[Geometry]) -> None:
    if len(left) != len(right):
        raise VectorizationError("Vectorized operations require sequences of equal length.")


def _validate_geometry_sequence(values: object, *, label: str) -> list[Geometry]:
    if isinstance(values, (str, bytes, bytearray)) or not isinstance(values, Sequence):
        raise VectorizationError(f"{label} must be a sequence of Geometry objects.")
    geometries = list(values)
    for index, geometry in enumerate(geometries):
        if not isinstance(geometry, Geometry):
            raise VectorizationError(
                f"{label}[{index}] must be a Geometry; received {type(geometry).__name__}."
            )
    return geometries


def _python_apply(
    func: Callable[[Geometry, Geometry], _T],
    left: Sequence[Geometry],
    right: Sequence[Geometry],
    *,
    kwargs: dict[str, object] | None = None,
) -> list[_T]:
    arguments = kwargs or {}
    return [func(a, b, **arguments) for a, b in zip(left, right)]


def _numpy_apply(
    func: Callable[[Geometry, Geometry], _T],
    left: Sequence[Geometry],
    right: Sequence[Geometry],
    *,
    kwargs: dict[str, object] | None = None,
    otypes: list[type[object]] | None = None,
) -> list[_T] | None:
    if _np is None:
        return None
    try:
        left_array = _np.asarray(left, dtype=object)  # type: ignore[attr-defined]
        right_array = _np.asarray(right, dtype=object)  # type: ignore[attr-defined]
        if getattr(left_array, "shape", None) != getattr(right_array, "shape", None):
            raise VectorizationError("Vectorized operations require sequences of equal length.")
        vectorized = _np.vectorize(lambda a, b: func(a, b, **(kwargs or {})), otypes=otypes)  # type: ignore[attr-defined]
        result = vectorized(left_array, right_array)
    except GeodistError:
        raise
    except Exception:
        return None
    if hasattr(result, "tolist"):
        return result.tolist()
    return list(result)


def _dispatch(
    func: Callable[[Geometry, Geometry], _T],
    left: object,
    right: object,
    *,
    kwargs: dict[str, object] | None = None,
    otypes: list[type[object]] | None = None,
) -> list[_T]:
    validated_left = _validate_geometry_sequence(left, label="left geometries")
    validated_right = _validate_geometry_sequence(right, label="right geometries")
    _ensure_lengths_match(validated_left, validated_right)
    numpy_result = _numpy_apply(func, validated_left, validated_right, kwargs=kwargs, otypes=otypes)
    if numpy_result is not None:
        return numpy_result
    return _python_apply(func, validated_left, validated_right, kwargs=kwargs)


def distance_many(
    left: Sequence[Geometry],
    right: Sequence[Geometry],
    *,
    crs: CRSLike = None,
) -> list[float]:
    """Compute pairwise distances across two geometry sequences."""
    return _dispatch(ops.distance, left, right, kwargs={"crs": crs}, otypes=[float])


def equals_many(left: Sequence[Geometry], right: Sequence[Geometry]) -> list[bool]:
    """Evaluate equality across two geometry sequences."""
    return _dispatch(ops.equals, left, right, otypes=[bool])


def intersects_many(left: Sequence[Geometry], right: Sequence[Geometry]) -> list[bool]:
    """Evaluate intersections across two geometry sequences."""
    return _dispatch(ops.intersects, left, right, otypes=[bool])


def within_many(left: Sequence[Geometry], right: Sequence[Geometry]) -> list[bool]:
    """Evaluate containment across two geometry sequences."""
    return _dispatch(ops.within, left, right, otypes=[bool])


__all__ = (
    "distance_many",
    "equals_many",
    "intersects_many",
    "within_many",
)
