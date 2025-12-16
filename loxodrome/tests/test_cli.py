from __future__ import annotations

from typer.testing import CliRunner

from loxodrome.cli import (
    DEFAULT_BOUNDING_BOX,
    DEFAULT_DESTINATION,
    DEFAULT_DESTINATION_3D,
    DEFAULT_ORIGIN,
    DEFAULT_ORIGIN_3D,
    DEFAULT_POINTS_A,
    DEFAULT_POINTS_B,
    app,
)

runner = CliRunner()


def test_geodesic_cli_with_bearings() -> None:
    result = runner.invoke(
        app,
        [
            "geodesic",
            "--origin",
            DEFAULT_ORIGIN,
            "--destination",
            DEFAULT_DESTINATION,
            "--bearings",
        ],
    )

    assert result.exit_code == 0
    assert "Great-circle (sphere) with bearings" in result.stdout
    assert "Distance: 4151828.428 m" in result.stdout
    assert "Initial bearing: 69.785 deg" in result.stdout
    assert "Final bearing: 101.602 deg" in result.stdout


def test_geodesic_cli_on_ellipsoid() -> None:
    result = runner.invoke(
        app,
        [
            "geodesic",
            "--origin",
            DEFAULT_ORIGIN,
            "--destination",
            DEFAULT_DESTINATION,
            "--ellipsoid",
        ],
    )

    assert result.exit_code == 0
    assert "Ellipsoidal geodesic distance" in result.stdout
    assert "Distance: 4161904.736 m" in result.stdout


def test_distance_3d_cli() -> None:
    result = runner.invoke(
        app,
        [
            "distance-3d",
            "--origin",
            DEFAULT_ORIGIN_3D,
            "--destination",
            DEFAULT_DESTINATION_3D,
        ],
    )

    assert result.exit_code == 0
    assert "ECEF chord distance" in result.stdout
    assert "Distance: 4088668.218 m" in result.stdout


def test_hausdorff_cli_outputs() -> None:
    result = runner.invoke(
        app,
        [
            "hausdorff",
            "--set-a",
            DEFAULT_POINTS_A,
            "--set-b",
            DEFAULT_POINTS_B,
        ],
    )

    assert result.exit_code == 0
    assert "Hausdorff without clipping" in result.stdout
    assert "Directed (A->B): 670661.854 m" in result.stdout
    assert "Directed (B->A): 1093215.274 m" in result.stdout
    assert "Symmetric Hausdorff: 1093215.274 m" in result.stdout


def test_hausdorff_cli_with_clipping() -> None:
    result = runner.invoke(
        app,
        [
            "hausdorff",
            "--set-a",
            DEFAULT_POINTS_A,
            "--set-b",
            DEFAULT_POINTS_B,
            "--clip",
            "--bounding-box",
            DEFAULT_BOUNDING_BOX,
        ],
    )

    assert result.exit_code == 0
    assert "Hausdorff with clipping" in result.stdout
    assert "Directed (A->B): 670661.854 m" in result.stdout
    assert "Symmetric Hausdorff: 1093215.274 m" in result.stdout
