# geodist-experiments

Experimentation, benchmarks, notebooks, and visualizations for geodist development.

This workspace is intentionally **not published** to PyPI. It holds exploratory code,
benchmarks, and tools that may eventually be promoted into the main `pygeodist` package.

## Setup

```bash
cd experiments && uv sync --all-extras
```

The `pygeodist` package is installed as an editable dependency, so changes to the core
library are reflected immediately.

## Usage

### Notebooks

```bash
cd experiments && uv run jupyter notebook notebooks/
```

### Benchmarks

```bash
cd experiments && uv run python -m experiments.bench_vectorized --count 100000
```

### Visualizations

Generate a PNG showing origin/destination pairs from a JSON file:

```bash
cd experiments
uv run python -m experiments.viz routes.example.json --output images/routes.png --theme dusk
```

Input files expect a top-level `routes` array with coordinates given as `[lat, lon]` or
`{"lat": ..., "lon": ...}` mappings (see `routes.example.json`); entries are validated
with Pydantic 2 for clearer errors.

You can also use the helpers directly:

```python
from experiments.viz import load_routes, render_routes, render_routes_figure
routes = load_routes("routes.example.json")
render_routes(routes, output="images/routes.png", theme="paper")
# Inline in a notebook:
fig = render_routes_figure(routes, theme="dusk")
fig
```

## Adding Dependencies

Feel free to add arbitrary dependencies here. This workspace is isolated from the
published PyPI package, so experimental or heavy dependencies won't affect users.

Edit `pyproject.toml` and run `uv sync` to install new packages.
