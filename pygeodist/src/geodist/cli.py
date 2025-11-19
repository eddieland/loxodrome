"""Development-only Typer CLI for quick local checks."""

from __future__ import annotations

import importlib.metadata

try:
    import typer
except ModuleNotFoundError as exc:  # pragma: no cover - exercised interactively
    # Provide a friendly error instead of a stack trace when the dev deps are missing.
    raise SystemExit(
        "The development CLI requires dev dependencies. "
        "Install them with `uv sync --group dev` before running this module."
    ) from exc


app = typer.Typer(
    add_completion=False,
    no_args_is_help=True,
    help="Development-only commands for inspecting the geodist bindings.",
)


def _extension_status() -> tuple[float | None, str | None]:
    """Return the earth radius in meters when the extension is available."""
    try:
        from ._geodist_rs import EARTH_RADIUS_METERS
    except ImportError as exc:
        return None, str(exc)
    return float(EARTH_RADIUS_METERS), None


@app.command()
def info() -> None:
    """Show package version and whether the Rust extension is importable."""
    version = importlib.metadata.version("pygeodist")
    radius_meters, error = _extension_status()
    typer.echo(f"pygeodist version: {version}")
    if radius_meters is None:
        typer.echo("Extension: not loaded")
        typer.echo(f"Import error: {error}")
        return

    typer.echo("Extension: loaded")
    typer.echo(f"Earth radius: {radius_meters:.3f} m")


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
    radius_meters, error = _extension_status()
    if radius_meters is None:
        typer.echo("Extension is not available; build it with `uv run maturin develop`.")
        if error:
            typer.echo(f"Import error: {error}")
        raise typer.Exit(code=1)

    normalized_unit = unit.lower()
    if normalized_unit in {"meter", "meters", "m"}:
        typer.echo(f"Earth radius: {radius_meters:.3f} m")
        return
    if normalized_unit in {"kilometer", "kilometers", "km"}:
        typer.echo(f"Earth radius: {radius_meters / 1000:.3f} km")
        return

    typer.echo("Unsupported unit. Choose 'meters' or 'kilometers'.")
    raise typer.Exit(code=1)


if __name__ == "__main__":  # pragma: no cover - manual entrypoint
    app()
