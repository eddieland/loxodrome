# flake8: noqa: PYI021

from typing import Final, Sized

EARTH_RADIUS_METERS: Final[float]

# Opaque handle returned by Rust geometry factories. Placeholder until full
# geometry bindings land; stored by Python wrappers to keep lifetimes stable.
GeometryHandle = Sized
