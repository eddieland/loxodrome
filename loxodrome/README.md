# loxodrome

Python bindings for `loxodrome`. The package surfaces the Rust-backed geometry
handles, geodesic distance/bearing helpers, and Hausdorff variants.

- Setup and publishing live in the repo-level `README.md`.
- The Typer CLI (`uv run lox info`) is a dev-only smoke check to confirm the
  extension loads.

## Shapely interoperability

Shapely is optional. Install the extra if you want to bridge loxodrome points
with Shapely:

```bash
pip install loxodrome[shapely]
```

Converters live in `loxodrome.ext.shapely` and keep imports guarded:

```python
from loxodrome import Point
from loxodrome.ext.shapely import from_shapely, to_shapely

point = Point(12.5, -45.0)
shapely_point = to_shapely(point)
round_tripped = from_shapely(shapely_point)
```

`Point`, `Point3D`, and `BoundingBox` are supported. Other geometry kinds raise
`TypeError`, and non-rectangular polygons raise `InvalidGeometryError`, until
matching kernels land.

## Demo notebook

A ready-to-run example lives at `experiments/notebooks/loxodrome_usage.ipynb`.
Launch it inside the uv-managed environment so imports resolve against the local build:

```bash
cd loxodrome
uv sync --all-extras
uv run maturin develop
uv run --with notebook jupyter notebook ../experiments/notebooks/loxodrome_usage.ipynb
```

For JupyterLab, swap the last line for
`uv run --with jupyterlab jupyter-lab ../experiments/notebooks/loxodrome_usage.ipynb`.
