# pygeodist

Python bindings for `geodist`. The Python package exposes the
`EARTH_RADIUS_METERS` constant, Rust-backed `Point` and `BoundingBox` wrappers,
`geodesic_distance`, `geodesic_with_bearings`, and Hausdorff helpers.

- For installation and development setup, see the project-level docs in `../README.md`.
- Publishing details (PyPI trusted publishing via GitHub Actions) also live in
  `../README.md#publishing`.

*This project was built from
[simple-modern-uv](https://github.com/jlevy/simple-modern-uv).*

## API scope and non-goals

- Public exports today: `EARTH_RADIUS_METERS`, `GeodistError`, `InvalidGeometryError`, `Point`, `BoundingBox`, `GeodesicResult`, `geodesic_distance`, `geodesic_with_bearings`, and Hausdorff (directed + clipped) helpers returning meters.
- Non-goals: mirroring Shapely's API, accepting free-form geometry tuples, or silently coercing unsupported shapes.
- Future Python surface (gated on Rust kernels, no promised timeline): witness point reporting, additional geometry wrappers, and vectorized helpers once Rust exposes them.
- The Typer CLI (`uv run geodist info`) is a dev-only helper to confirm the extension loads.

## Shapely interoperability

Shapely is optional. Install the extra if you want to bridge geodist points
with Shapely:

```bash
pip install pygeodist[shapely]
```

Converters live in `geodist.ext.shapely` and keep imports guarded:

```python
from geodist import Point
from geodist.ext.shapely import from_shapely, to_shapely

point = Point(12.5, -45.0)
shapely_point = to_shapely(point)
round_tripped = from_shapely(shapely_point)
```

Only `Point` is supported for now; other geometry kinds raise `InvalidGeometryError`
until the Rust kernels provide matching types.

## Running the demo notebook

A ready-to-run example lives at `notebooks/geodist_usage.ipynb`. Launch it inside
the uv-managed environment so imports resolve against the local build:

```bash
cd pygeodist
uv sync --all-extras
uv run maturin develop
uv run --with notebook jupyter notebook notebooks/geodist_usage.ipynb
```

If you prefer JupyterLab, swap the last line for
`uv run --with jupyterlab jupyter-lab notebooks/geodist_usage.ipynb`.
