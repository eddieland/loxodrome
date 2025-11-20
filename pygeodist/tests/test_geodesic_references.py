"""GeographicLib reference fixtures for ellipsoidal geodesics on WGS84."""

from __future__ import annotations

from dataclasses import dataclass

import pytest
from pytest import approx

from geodist import Ellipsoid, Point, geodesic_distance_on_ellipsoid, geodesic_with_bearings_on_ellipsoid


@dataclass(slots=True)
class ReferenceCase:
    name: str
    origin: tuple[float, float]
    destination: tuple[float, float]
    distance_m: float
    initial_bearing_deg: float
    final_bearing_deg: float


# Values generated via GeographicLib 2.0 (Karney) using Geodesic.WGS84.Inverse.
REFERENCE_CASES = [
    ReferenceCase(
        name="nyc_london",
        origin=(40.7128, -74.0060),
        destination=(51.5074, -0.1278),
        distance_m=5_585_233.578_931_3,
        initial_bearing_deg=51.241_229_119_512_35,
        final_bearing_deg=108.368_998_113_182_64,
    ),
    ReferenceCase(
        name="almost_antipodal",
        origin=(0.0, 0.0),
        destination=(-0.5, 179.5),
        distance_m=19_936_288.578_965_314,
        initial_bearing_deg=154.328_127_131_708_13,
        final_bearing_deg=25.672_914_530_058_396,
    ),
    ReferenceCase(
        name="polar_cross",
        origin=(89.0, 0.0),
        destination=(85.0, 90.0),
        distance_m=569_487.910_026_2804,
        initial_bearing_deg=78.718_341_086_595_79,
        final_bearing_deg=168.674_679_425_412_28,
    ),
    ReferenceCase(
        name="short_haul_san_francisco",
        origin=(37.7749, -122.4194),
        destination=(37.7750, -122.4185),
        distance_m=80.063_255_017_781_93,
        initial_bearing_deg=82.031_107_905_538_65,
        final_bearing_deg=82.031_659_210_920_5,
    ),
]


@pytest.mark.parametrize("case", REFERENCE_CASES, ids=lambda case: case.name)
def test_ellipsoidal_distance_and_bearings_match_reference(case: ReferenceCase) -> None:
    origin = Point(*case.origin)
    destination = Point(*case.destination)
    wgs84 = Ellipsoid.wgs84()

    result = geodesic_with_bearings_on_ellipsoid(origin, destination, wgs84)
    assert result.distance_m == approx(case.distance_m, abs=1e-6)
    assert result.initial_bearing_deg == approx(case.initial_bearing_deg, abs=5e-8)
    assert result.final_bearing_deg == approx(case.final_bearing_deg, abs=5e-8)

    distance_only = geodesic_distance_on_ellipsoid(origin, destination, wgs84)
    assert distance_only == approx(case.distance_m, abs=1e-6)
