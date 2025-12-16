"""Development-only Typer CLI for quick local checks and demos."""

from __future__ import annotations

import importlib.metadata
import importlib.util
from typing import TYPE_CHECKING

from loxodrome import (
    EARTH_RADIUS_METERS,
    BoundingBox,
    Point,
    Point3D,
    geodesic_distance,
    geodesic_distance_3d,
    geodesic_distance_on_ellipsoid,
    geodesic_with_bearings,
    geodesic_with_bearings_on_ellipsoid,
    hausdorff_clipped,
    hausdorff_directed,
    hausdorff_directed_clipped,
)
from loxodrome import hausdorff as hausdorff_sym

try:
    import typer
except ModuleNotFoundError as exc:  # pragma: no cover - exercised interactively
    # Provide a friendly error instead of a stack trace when the dev deps are missing.
    raise SystemExit(
        "The development CLI requires dev dependencies. "
        "Install them with `uv sync --group dev` before running this module."
    ) from exc

if TYPE_CHECKING:  # pragma: no cover - import guard for type checking only
    from .geometry import BoundingBox, Point, Point3D
    from .ops import HausdorffDirectedWitness


DEFAULT_ORIGIN = "37.6189,-122.3750"  # San Francisco International Airport
DEFAULT_DESTINATION = "40.6413,-73.7781"  # John F. Kennedy International Airport
DEFAULT_ORIGIN_3D = "37.6189,-122.3750,10.0"
DEFAULT_DESTINATION_3D = "40.6413,-73.7781,20.0"
DEFAULT_POINTS_A = "37.7749,-122.4194;34.0522,-118.2437"  # San Francisco and Los Angeles
DEFAULT_POINTS_B = "36.1699,-115.1398;47.6062,-122.3321"  # Las Vegas and Seattle
DEFAULT_BOUNDING_BOX = "30.0,50.0,-130.0,-110.0"


app = typer.Typer(
    add_completion=False,
    no_args_is_help=True,
    help="Development-only commands for inspecting the loxodrome bindings.",
)


@app.command()
def info() -> None:
    """Show package version and whether the Rust extension is importable."""
    version = importlib.metadata.version("loxodrome")
    typer.echo(f"loxodrome version: {version}")
    typer.echo(f"Shapely interop helpers: {_shapely_interop_status()}")
    typer.echo("Extension: loaded")
    typer.echo(f"Earth radius: {EARTH_RADIUS_METERS:.3f} m")


@app.command()
def earth_radius(
    unit: str = typer.Option(
        "meters",
        "--unit",
        "-u",
        help="Unit to display; supports 'meters' or 'kilometers'.",
        case_sensitive=False,
    ),
) -> None:
    """Print the earth radius using the compiled constant."""
    normalized_unit = unit.lower()
    if normalized_unit in {"meter", "meters", "m"}:
        typer.echo(f"Earth radius: {EARTH_RADIUS_METERS:.3f} m")
        return
    if normalized_unit in {"kilometer", "kilometers", "km"}:
        typer.echo(f"Earth radius: {EARTH_RADIUS_METERS / 1000:.3f} km")
        return

    typer.echo("Unsupported unit. Choose 'meters' or 'kilometers'.")
    raise typer.Exit(code=1)


@app.command()
def geodesic(
    origin: str = typer.Option(
        DEFAULT_ORIGIN,
        "--origin",
        "-o",
        help="Origin point as 'lat,lon'.",
    ),
    destination: str = typer.Option(
        DEFAULT_DESTINATION,
        "--destination",
        "-d",
        help="Destination point as 'lat,lon'.",
    ),
    ellipsoid: bool = typer.Option(
        False,
        "--ellipsoid",
        help="Use ellipsoidal geodesics instead of the mean-radius sphere.",
    ),
    bearings: bool = typer.Option(False, "--bearings", help="Include forward and reverse bearings."),
) -> None:
    """Compute a geodesic distance (and optional bearings) between two points."""
    origin_point = _parse_point(origin)
    destination_point = _parse_point(destination)
    if ellipsoid:
        if bearings:
            result = geodesic_with_bearings_on_ellipsoid(origin_point, destination_point)
            typer.echo("Ellipsoidal geodesic with bearings")
            typer.echo(f"Distance: {result.distance_m:.3f} m")
            typer.echo(f"Initial bearing: {result.initial_bearing_deg:.3f} deg")
            typer.echo(f"Final bearing: {result.final_bearing_deg:.3f} deg")
            return

        distance = geodesic_distance_on_ellipsoid(origin_point, destination_point)
        typer.echo("Ellipsoidal geodesic distance")
        typer.echo(f"Distance: {distance:.3f} m")
        return

    if bearings:
        result = geodesic_with_bearings(origin_point, destination_point)
        typer.echo("Great-circle (sphere) with bearings")
        typer.echo(f"Distance: {result.distance_m:.3f} m")
        typer.echo(f"Initial bearing: {result.initial_bearing_deg:.3f} deg")
        typer.echo(f"Final bearing: {result.final_bearing_deg:.3f} deg")
        return

    distance = geodesic_distance(origin_point, destination_point)
    typer.echo("Great-circle (sphere) distance")
    typer.echo(f"Distance: {distance:.3f} m")


@app.command(name="distance-3d")
def distance_3d(
    origin: str = typer.Option(
        DEFAULT_ORIGIN_3D,
        "--origin",
        "-o",
        help="Origin point as 'lat,lon,alt_m'.",
    ),
    destination: str = typer.Option(
        DEFAULT_DESTINATION_3D,
        "--destination",
        "-d",
        help="Destination point as 'lat,lon,alt_m'.",
    ),
) -> None:
    """Compute the straight-line (ECEF chord) distance between two 3D points."""
    origin_point = _parse_point3d(origin)
    destination_point = _parse_point3d(destination)
    distance = geodesic_distance_3d(origin_point, destination_point)
    typer.echo("ECEF chord distance")
    typer.echo(f"Distance: {distance:.3f} m")


@app.command()
def hausdorff(
    set_a: str = typer.Option(
        DEFAULT_POINTS_A,
        "--set-a",
        "-a",
        help="Semicolon-delimited list of 'lat,lon' pairs for set A.",
    ),
    set_b: str = typer.Option(
        DEFAULT_POINTS_B,
        "--set-b",
        "-b",
        help="Semicolon-delimited list of 'lat,lon' pairs for set B.",
    ),
    clip: bool = typer.Option(
        False,
        "--clip",
        help="Clip both sets to the bounding box before computing distances.",
    ),
    bounding_box: str = typer.Option(
        DEFAULT_BOUNDING_BOX,
        "--bounding-box",
        "-B",
        help="Bounding box as 'min_lat,max_lat,min_lon,max_lon' for clipping runs.",
    ),
) -> None:
    """Compute directed and symmetric Hausdorff distances between two point sets."""
    points_a = _parse_points(set_a)
    points_b = _parse_points(set_b)
    if clip:
        bbox = _parse_bounding_box(bounding_box)
        directed = hausdorff_directed_clipped(points_a, points_b, bbox)
        symmetric = hausdorff_clipped(points_a, points_b, bbox)
        typer.echo(
            "Hausdorff with clipping to "
            f"({bbox.min_lat:.4f}, {bbox.min_lon:.4f})..({bbox.max_lat:.4f}, {bbox.max_lon:.4f})"
        )
    else:
        directed = hausdorff_directed(points_a, points_b)
        symmetric = hausdorff_sym(points_a, points_b)
        typer.echo("Hausdorff without clipping")

    typer.echo(_format_directed_witness("Directed (A->B)", directed))
    typer.echo(_format_directed_witness("Directed (B->A)", symmetric.b_to_a))
    typer.echo(f"Symmetric Hausdorff: {symmetric.distance_m:.3f} m")


@app.command()
def demo() -> None:
    """Run a quick tour of the bindings using sample coordinates."""
    typer.echo("=== Geodesic on sphere with bearings ===")
    geodesic(bearings=True)
    typer.echo()
    typer.echo("=== Geodesic on ellipsoid ===")
    geodesic(ellipsoid=True)
    typer.echo()
    typer.echo("=== 3D chord distance ===")
    distance_3d()
    typer.echo()
    typer.echo("=== Hausdorff demo (with clipping) ===")
    hausdorff(clip=True)


def _shapely_interop_status() -> str:
    """Return a short status string for optional Shapely helpers."""
    return "available" if importlib.util.find_spec("shapely.geometry") else "not installed"


def _parse_point(serialized: str) -> Point:
    """Parse a "lat,lon" string into a :class:`loxodrome.geometry.Point`."""
    try:
        lat_str, lon_str = (component.strip() for component in serialized.split(",", maxsplit=1))
        latitude = float(lat_str)
        longitude = float(lon_str)
    except ValueError as exc:
        raise typer.BadParameter("Point must be formatted as 'lat,lon'.") from exc
    return Point(latitude, longitude)


def _parse_point3d(serialized: str) -> Point3D:
    """Parse a "lat,lon,alt_m" string into a :class:`loxodrome.geometry.Point3D`."""
    try:
        lat_str, lon_str, altitude_str = (component.strip() for component in serialized.split(",", maxsplit=2))
        latitude = float(lat_str)
        longitude = float(lon_str)
        altitude_m = float(altitude_str)
    except ValueError as exc:
        raise typer.BadParameter("Point3D must be formatted as 'lat,lon,alt_m'.") from exc
    return Point3D(latitude, longitude, altitude_m)


def _parse_points(serialized: str) -> list[Point]:
    """Parse a semicolon-delimited list of "lat,lon" strings into :class:`Point` objects."""
    points = [_parse_point(chunk) for chunk in serialized.split(";") if chunk.strip()]
    if not points:
        raise typer.BadParameter("Provide at least one point as 'lat,lon' pair.")
    return points


def _parse_bounding_box(serialized: str) -> BoundingBox:
    """Parse "min_lat,max_lat,min_lon,max_lon" into a :class:`BoundingBox`."""
    try:
        min_lat_str, max_lat_str, min_lon_str, max_lon_str = (
            component.strip() for component in serialized.split(",", maxsplit=3)
        )
        min_lat = float(min_lat_str)
        max_lat = float(max_lat_str)
        min_lon = float(min_lon_str)
        max_lon = float(max_lon_str)
    except ValueError as exc:
        raise typer.BadParameter("Bounding box must be formatted as 'min_lat,max_lat,min_lon,max_lon'.") from exc
    return BoundingBox(min_lat, max_lat, min_lon, max_lon)


def _format_directed_witness(label: str, witness: "HausdorffDirectedWitness") -> str:
    return (
        f"{label}: {witness.distance_m:.3f} m (origin #{witness.origin_index} -> candidate #{witness.candidate_index})"
    )


if __name__ == "__main__":  # pragma: no cover - manual entrypoint
    app()
