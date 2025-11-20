# pygeodist

Python bindings for `geodist`. The Python package currently only exposes the
`EARTH_RADIUS_METERS` constant from the Rust core while the geodesic kernels are
built out; new bindings will be added as the Rust surface stabilizes.

- For installation and development setup, see the project-level docs in `../README.md`.
- Publishing details (PyPI trusted publishing via GitHub Actions) also live in
  `../README.md#publishing`.

*This project was built from
[simple-modern-uv](https://github.com/jlevy/simple-modern-uv).*

## API scope and non-goals

- Public exports today: `EARTH_RADIUS_METERS`, `GeodistError`, `InvalidGeometryError`, and the `Point` wrapper once the extension is built.
- Non-goals: mirroring Shapely's API, accepting free-form geometry tuples, or silently coercing unsupported shapes.
- Future Python surface (gated on Rust kernels, no promised timeline): wrappers for geodesic distance/bearings, Hausdorff (including bounding-box-clipped variants), and additional geometry wrappers once Rust exposes them.
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
