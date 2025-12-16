"""Image-based visualizations for loxodrome experiment results."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence

import matplotlib.pyplot as plt
import numpy as np
from matplotlib.colors import LinearSegmentedColormap
from pydantic import BaseModel, ConfigDict, model_validator

from loxodrome import ops
from loxodrome.geometry import Point

__all__ = ["RouteResult", "load_routes", "render_routes", "render_routes_figure", "main"]


@dataclass(frozen=True)
class RouteResult:
    """Container for a single origin/destination result."""

    origin: Point
    destination: Point
    distance_km: float
    label: str | None = None


class _Coordinate(BaseModel):
    lat: float
    lon: float

    model_config = ConfigDict(extra="forbid")

    @model_validator(mode="before")
    @classmethod
    def _coerce_sequence(cls, value: object) -> object:
        if isinstance(value, dict):
            return value
        if isinstance(value, (list, tuple)) and len(value) == 2:
            return {"lat": value[0], "lon": value[1]}
        return value


class _RouteResource(BaseModel):
    origin: _Coordinate
    destination: _Coordinate
    label: str | None = None
    distance_km: float | None = None

    model_config = ConfigDict(extra="forbid")


class _RouteCollection(BaseModel):
    routes: list[_RouteResource]

    model_config = ConfigDict(extra="forbid")


def load_routes(source: str | Path) -> list[RouteResult]:
    """Load route results from a JSON file.

    The file should contain a top-level object with a ``routes`` array. Each route
    entry must define ``origin`` and ``destination`` coordinates, provided either as
    ``[lat, lon]`` lists or ``{"lat": ..., "lon": ...}`` mappings. ``distance_km`` is
    optional; missing values are computed with ``loxodrome.ops.geodesic_distance``.
    """
    raw_text = Path(source).read_text()
    parsed = _RouteCollection.model_validate_json(raw_text)

    routes: list[RouteResult] = []
    for entry in parsed.routes:
        origin = Point(entry.origin.lat, entry.origin.lon)
        destination = Point(entry.destination.lat, entry.destination.lon)
        distance_km = entry.distance_km
        if distance_km is None:
            distance_km = ops.geodesic_distance(origin, destination) / 1000.0
        routes.append(RouteResult(origin=origin, destination=destination, distance_km=distance_km, label=entry.label))
    return routes


def render_routes(
    routes: Sequence[RouteResult],
    *,
    output: str | Path,
    title: str = "Loxodrome routes",
    dpi: int = 240,
    theme: str = "dusk",
    show_labels: bool = True,
) -> Path:
    """Render origin/destination routes to a PNG image."""
    fig = render_routes_figure(
        routes,
        title=title,
        dpi=dpi,
        theme=theme,
        show_labels=show_labels,
    )
    output_path = Path(output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(output_path, dpi=dpi, facecolor=fig.get_facecolor())
    plt.close(fig)
    return output_path


def render_routes_figure(
    routes: Sequence[RouteResult],
    *,
    title: str = "Loxodrome routes",
    dpi: int = 240,
    theme: str = "dusk",
    show_labels: bool = True,
) -> plt.Figure:
    """Create a matplotlib figure for the given routes without writing to disk."""
    if not routes:
        raise ValueError("At least one route is required to render a visualization")
    style = _THEMES.get(theme)
    if style is None:
        raise ValueError(f"Unknown theme '{theme}'. Available themes: {', '.join(sorted(_THEMES))}")

    lats, lons = _collect_extents(routes)
    lat_padding = (max(lats) - min(lats)) * 0.1 or 5.0
    lon_padding = (max(lons) - min(lons)) * 0.1 or 5.0
    lat_min, lat_max = min(lats) - lat_padding, max(lats) + lat_padding
    lon_min, lon_max = min(lons) - lon_padding, max(lons) + lon_padding

    fig, ax = plt.subplots(figsize=(8, 5), dpi=dpi, layout="constrained", facecolor=style["background"])
    ax.set_facecolor(style["background"])
    _apply_gradient(ax, lon_min, lon_max, lat_min, lat_max, style)

    for route in routes:
        lat_points = (route.origin.lat, route.destination.lat)
        lon_points = (route.origin.lon, route.destination.lon)
        ax.plot(
            lon_points,
            lat_points,
            color=style["line"],
            linewidth=2.5,
            alpha=0.85,
            solid_capstyle="round",
        )
        ax.scatter(
            lon_points,
            lat_points,
            s=70,
            zorder=3,
            linewidth=0,
            c=(style["point_origin"], style["point_destination"]),
        )

        if show_labels:
            midpoint_lat = sum(lat_points) / 2
            midpoint_lon = sum(lon_points) / 2
            label = route.label or f"{route.distance_km:.1f} km"
            ax.text(
                midpoint_lon,
                midpoint_lat,
                label,
                color=style["text"],
                fontsize=10,
                fontweight="semibold",
                ha="center",
                va="center",
                bbox=dict(
                    boxstyle="round,pad=0.25",
                    facecolor=style["label_bg"],
                    edgecolor="none",
                    alpha=0.65,
                ),
            )

    ax.set_xlim(lon_min, lon_max)
    ax.set_ylim(lat_min, lat_max)
    ax.set_xlabel("Longitude (deg)", color=style["text"])
    ax.set_ylabel("Latitude (deg)", color=style["text"])
    ax.set_title(title, color=style["text"], fontsize=14, fontweight="bold")
    _stylize_axes(ax, style)

    return fig


def main(argv: Sequence[str] | None = None) -> None:
    """CLI entry point for rendering a routes JSON file to a PNG."""
    parser = argparse.ArgumentParser(description="Render loxodrome results to a pretty PNG.")
    parser.add_argument("input", type=Path, help="Path to a JSON file containing a 'routes' array")
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("route-visualization.png"),
        help="Where to write the rendered image (default: route-visualization.png)",
    )
    parser.add_argument(
        "--theme",
        choices=sorted(_THEMES),
        default="dusk",
        help="Color theme to use for the visualization",
    )
    parser.add_argument("--title", default="Loxodrome routes", help="Title to display at the top of the figure")
    parser.add_argument("--dpi", type=int, default=240, help="DPI for the saved image")
    parser.add_argument(
        "--hide-labels",
        action="store_true",
        help="Disable per-route labels; distances will still be computed",
    )
    args = parser.parse_args(argv)

    routes = load_routes(args.input)
    output_path = render_routes(
        routes,
        output=args.output,
        title=args.title,
        dpi=args.dpi,
        theme=args.theme,
        show_labels=not args.hide_labels,
    )
    print(f"Wrote visualization to {output_path.resolve()}")


def _collect_extents(routes: Iterable[RouteResult]) -> tuple[list[float], list[float]]:
    latitudes: list[float] = []
    longitudes: list[float] = []
    for route in routes:
        latitudes.extend((route.origin.lat, route.destination.lat))
        longitudes.extend((route.origin.lon, route.destination.lon))
    return latitudes, longitudes


def _apply_gradient(ax: plt.Axes, lon_min: float, lon_max: float, lat_min: float, lat_max: float, style: dict[str, str]) -> None:
    gradient = np.linspace(0, 1, 256)
    gradient = np.vstack((gradient, gradient))
    cmap = LinearSegmentedColormap.from_list("loxodrome-viz-bg", [style["gradient_bottom"], style["gradient_top"]])
    ax.imshow(
        gradient,
        extent=[lon_min, lon_max, lat_min, lat_max],
        origin="lower",
        cmap=cmap,
        alpha=0.9,
        aspect="auto",
        zorder=0,
    )


def _stylize_axes(ax: plt.Axes, style: dict[str, str]) -> None:
    for spine in ax.spines.values():
        spine.set_color(style["accent"])
        spine.set_linewidth(1.0)
    ax.tick_params(colors=style["text"], labelsize=10)
    ax.grid(color=style["grid"], linewidth=0.6, alpha=0.4)


_THEMES: dict[str, dict[str, str]] = {
    "dusk": {
        "background": "#0b0c10",
        "gradient_bottom": "#111827",
        "gradient_top": "#1f2937",
        "line": "#60a5fa",
        "point_origin": "#f59e0b",
        "point_destination": "#34d399",
        "text": "#e5e7eb",
        "grid": "#9ca3af",
        "label_bg": "#1f2937",
        "accent": "#374151",
    },
    "paper": {
        "background": "#f7f7f2",
        "gradient_bottom": "#f0efeb",
        "gradient_top": "#dcd7c9",
        "line": "#6b705c",
        "point_origin": "#cb997e",
        "point_destination": "#386641",
        "text": "#1f2933",
        "grid": "#b7b7a4",
        "label_bg": "#fffefb",
        "accent": "#8a817c",
    },
}


if __name__ == "__main__":
    main()
