from __future__ import annotations

import types
from collections.abc import Mapping
from typing import Any

import pytest

from geodist import io
from geodist.errors import (
    CRSValidationError,
    GeometryTypeError,
    InvalidGeometryError,
    KernelExecutionError,
    KernelUnavailableError,
)
from geodist.geometry import Geometry


class _DummyHandle:
    def __init__(self, value: object) -> None:
        self.value = value

    def __len__(self) -> int:
        return 1


def test_wkt_round_trip_passes_crs_through(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: dict[str, object] = {}

    def loads_kernel(wkt: str, *, crs: object = None) -> _DummyHandle:
        calls["loads"] = (wkt, crs)
        return _DummyHandle(wkt)

    def dumps_kernel(handle: _DummyHandle, *, crs: object = None) -> str:
        calls["dumps"] = (handle.value, crs)
        return f"WKT:{handle.value}"

    monkeypatch.setattr(io, "_geodist_rs", types.SimpleNamespace(loads_wkt=loads_kernel, dumps_wkt=dumps_kernel))

    geometry = io.loads_wkt("POINT (1 2)", crs="EPSG:4326")

    assert isinstance(geometry, Geometry)
    assert geometry.crs == "EPSG:4326"
    assert calls["loads"] == ("POINT (1 2)", "EPSG:4326")
    assert io.dumps_wkt(geometry) == "WKT:POINT (1 2)"
    assert calls["dumps"] == ("POINT (1 2)", "EPSG:4326")


def test_wkb_accepts_bytes_like_and_returns_bytes(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: dict[str, object] = {}

    def loads_kernel(wkb: bytes, *, crs: object = None) -> _DummyHandle:
        calls["loads"] = (wkb, crs)
        return _DummyHandle(int.from_bytes(wkb, "big"))

    def dumps_kernel(handle: _DummyHandle, *, crs: object = None) -> bytes:
        calls["dumps"] = (handle.value, crs)
        return bytes([handle.value])  # type: ignore[list-item]

    monkeypatch.setattr(io, "_geodist_rs", types.SimpleNamespace(loads_wkb=loads_kernel, dumps_wkb=dumps_kernel))

    geometry = io.loads_wkb(memoryview(b"\x05"), crs=4326)

    assert geometry.crs == 4326
    assert calls["loads"] == (b"\x05", 4326)
    assert io.dumps_wkb(geometry) == b"\x05"
    assert calls["dumps"] == (5, 4326)


def test_geojson_crs_fallback(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: dict[str, object] = {}

    def loads_kernel(mapping: object, *, crs: object = None) -> _DummyHandle:
        calls["loads"] = (mapping, crs)
        return _DummyHandle(mapping)

    def dumps_kernel(handle: _DummyHandle, *, crs: object = None) -> dict[str, object]:
        calls["dumps"] = (handle.value, crs)
        return {"type": "Point", "coordinates": [0.0, 1.0], "crs": crs}

    monkeypatch.setattr(
        io,
        "_geodist_rs",
        types.SimpleNamespace(loads_geojson=loads_kernel, dumps_geojson=dumps_kernel),
    )

    geojson: Mapping[str, Any] = {"type": "Point", "coordinates": [0.0, 1.0], "crs": "EPSG:3857"}
    geometry = io.loads_geojson(geojson)

    assert geometry.crs == "EPSG:3857"
    assert calls["loads"][1] == "EPSG:3857"  # type: ignore[index]
    exported = io.dumps_geojson(geometry)
    assert exported["crs"] == "EPSG:3857"
    assert calls["dumps"][1] == "EPSG:3857"  # type: ignore[index]


def test_invalid_inputs_and_kernel_failures(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(io, "_geodist_rs", types.SimpleNamespace())

    with pytest.raises(InvalidGeometryError):
        io.loads_wkt(123)  # type: ignore[arg-type]

    with pytest.raises(InvalidGeometryError):
        io.loads_geojson(123)  # type: ignore[arg-type]

    with pytest.raises(CRSValidationError):
        io.loads_wkb(b"", crs=object())  # type: ignore[arg-type]

    with pytest.raises(KernelUnavailableError):
        io.loads_wkb(b"00")

    geometry = Geometry._from_handle(_DummyHandle("value"))

    with pytest.raises(GeometryTypeError):
        io.dumps_geojson(object())  # type: ignore[arg-type]

    def bad_dumps_kernel(handle: _DummyHandle, *, crs: object = None) -> int:
        return 1

    monkeypatch.setattr(
        io,
        "_geodist_rs",
        types.SimpleNamespace(loads_wkt=lambda value, *, crs=None: _DummyHandle(value), dumps_wkt=bad_dumps_kernel),
    )
    geometry = io.loads_wkt("POINT (0 0)")
    with pytest.raises(KernelExecutionError):
        io.dumps_wkt(geometry)
