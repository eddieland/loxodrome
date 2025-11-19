from __future__ import annotations

import types

import pytest

from geodist import vectorized
from geodist import ops
from geodist.errors import VectorizationError
from geodist.geometry import Point


class _DummyHandle:
    def __init__(self, value: float) -> None:
        self.value = value

    def __len__(self) -> int:
        return 1


def _point_with_handle(value: float) -> Point:
    return Point._from_handle(_DummyHandle(value))


def test_length_mismatch_raises_vectorization_error() -> None:
    left = [_point_with_handle(1.0)]
    right = [_point_with_handle(1.0), _point_with_handle(2.0)]

    with pytest.raises(VectorizationError):
        vectorized.distance_many(left, right)


def test_python_fallback_when_numpy_missing(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: list[tuple[float, float, object | None]] = []

    def distance_kernel(left: _DummyHandle, right: _DummyHandle, *, crs: object = None) -> float:
        calls.append((left.value, right.value, crs))
        return left.value + right.value

    monkeypatch.setattr(vectorized, "_np", None)
    monkeypatch.setattr(ops, "_geodist_rs", types.SimpleNamespace(distance=distance_kernel))

    left = [_point_with_handle(1.0), _point_with_handle(2.5)]
    right = [_point_with_handle(0.0), _point_with_handle(0.5)]

    assert vectorized.distance_many(left, right, crs=4326) == [1.0, 3.0]
    assert calls == [(1.0, 0.0, 4326), (2.5, 0.5, 4326)]


def test_numpy_fast_path_is_used_when_available(monkeypatch: pytest.MonkeyPatch) -> None:
    class _FakeArray(list[Point]):
        @property
        def shape(self) -> tuple[int]:
            return (len(self),)

    class _FakeNumpy:
        def __init__(self) -> None:
            self.vectorize_calls = 0

        def asarray(self, values: object, *, dtype: object | None = None) -> _FakeArray:  # noqa: ARG002
            return _FakeArray(values)  # type: ignore[arg-type]

        def vectorize(self, func, otypes=None):  # type: ignore[no-untyped-def]  # noqa: ANN001, ANN204, ARG002
            self.vectorize_calls += 1

            def wrapper(left: _FakeArray, right: _FakeArray):
                return [func(l, r) for l, r in zip(left, right)]

            return wrapper

    fake_numpy = _FakeNumpy()
    monkeypatch.setattr(vectorized, "_np", fake_numpy)

    def equals_kernel(left: _DummyHandle, right: _DummyHandle) -> bool:
        return left.value == right.value

    monkeypatch.setattr(ops, "_geodist_rs", types.SimpleNamespace(equals=equals_kernel))

    left = [_point_with_handle(1.0), _point_with_handle(2.0)]
    right = [_point_with_handle(1.0), _point_with_handle(3.0)]

    assert vectorized.equals_many(left, right) == [True, False]
    assert fake_numpy.vectorize_calls == 1


def test_invalid_inputs_raise_vectorization_error() -> None:
    with pytest.raises(VectorizationError):
        vectorized.equals_many("bad input", [])  # type: ignore[arg-type]

    with pytest.raises(VectorizationError):
        vectorized.intersects_many([_point_with_handle(1.0), object()], [])  # type: ignore[list-item]
