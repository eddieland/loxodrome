from __future__ import annotations

import types

import pytest

from geodist import ops
from geodist.errors import (
    CRSValidationError,
    GeometryTypeError,
    InvalidGeometryError,
    KernelUnavailableError,
)
from geodist.geometry import Geometry, Point


class _DummyHandle:
    def __init__(self, value: float) -> None:
        self.value = value

    def __len__(self) -> int:
        return 1


def _point_with_handle(value: float, crs: int | str | None = None) -> Point:
    return Point._from_handle(_DummyHandle(value), crs=crs)


def test_distance_dispatches_to_kernel(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: dict[str, object] = {}

    def kernel(left: _DummyHandle, right: _DummyHandle, *, crs: object = None) -> float:
        calls["args"] = (left.value, right.value, crs)
        return left.value + right.value

    monkeypatch.setattr(ops, "_geodist_rs", types.SimpleNamespace(distance=kernel))

    a = _point_with_handle(1.0, crs="EPSG:4326")
    b = _point_with_handle(2.0, crs="EPSG:4326")

    assert ops.distance(a, b, crs="EPSG:4326") == 3.0
    assert calls["args"] == (1.0, 2.0, "EPSG:4326")


def test_predicates_return_booleans(monkeypatch: pytest.MonkeyPatch) -> None:
    def equals_kernel(left: _DummyHandle, right: _DummyHandle) -> bool:
        return left.value == right.value

    def intersects_kernel(left: _DummyHandle, right: _DummyHandle) -> bool:
        return left.value <= right.value

    monkeypatch.setattr(
        ops,
        "_geodist_rs",
        types.SimpleNamespace(equals=equals_kernel, intersects=intersects_kernel, within=intersects_kernel),
    )

    a = _point_with_handle(1.0)
    b = _point_with_handle(2.0)
    c = _point_with_handle(1.0)

    assert ops.equals(a, b) is False
    assert ops.equals(a, c) is True
    assert ops.intersects(a, b) is True
    assert ops.within(a, b) is True


def test_centroid_wraps_handle(monkeypatch: pytest.MonkeyPatch) -> None:
    def centroid_kernel(handle: _DummyHandle) -> _DummyHandle:
        return _DummyHandle(handle.value / 2)

    monkeypatch.setattr(ops, "_geodist_rs", types.SimpleNamespace(centroid=centroid_kernel))

    geometry = _point_with_handle(6.0, crs=4326)
    centroid = ops.centroid(geometry)

    assert isinstance(centroid, Point)
    assert centroid.crs == 4326
    assert isinstance(centroid._handle, _DummyHandle)  # type: ignore[attr-defined]
    assert centroid._handle.value == 3.0  # type: ignore[attr-defined]


def test_buffer_returns_geometry_wrapped_handle(monkeypatch: pytest.MonkeyPatch) -> None:
    def buffer_kernel(handle: _DummyHandle, distance_meters: float) -> _DummyHandle:
        return _DummyHandle(handle.value + distance_meters)

    monkeypatch.setattr(ops, "_geodist_rs", types.SimpleNamespace(buffer=buffer_kernel))

    geometry = _point_with_handle(1.0, crs="EPSG:3857")
    buffered = ops.buffer(geometry, 2.0)

    assert isinstance(buffered, Geometry)
    assert buffered.crs == "EPSG:3857"
    assert isinstance(buffered._handle, _DummyHandle)  # type: ignore[attr-defined]
    assert buffered._handle.value == 3.0  # type: ignore[attr-defined]


def test_invalid_inputs_raise_consistent_errors(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(ops, "_geodist_rs", types.SimpleNamespace())
    geometry = _point_with_handle(1.0)

    with pytest.raises(GeometryTypeError):
        ops.distance(object(), geometry)

    with pytest.raises(CRSValidationError):
        ops.distance(geometry, geometry, crs=object())  # type: ignore[arg-type]

    with pytest.raises(InvalidGeometryError):
        ops.buffer(geometry, "bad")  # type: ignore[arg-type]

    with pytest.raises(KernelUnavailableError):
        ops.equals(geometry, geometry)


def test_kernel_validation_errors_are_mapped(monkeypatch: pytest.MonkeyPatch) -> None:
    def distance_kernel(left: _DummyHandle, right: _DummyHandle, *, crs: object = None) -> float:
        raise ValueError("invalid geometry")

    monkeypatch.setattr(ops, "_geodist_rs", types.SimpleNamespace(distance=distance_kernel))

    geometry = _point_with_handle(1.0)

    with pytest.raises(InvalidGeometryError):
        ops.distance(geometry, geometry)
