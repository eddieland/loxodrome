# Repository Guidelines

## Project Structure & Modules

- Root contains Rust crate `geodist-rs` (core kernels) and Python package `pygeodist` (bindings and tooling).
- Rust: see `geodist-rs/src`; binary entry in `src/main.rs` for now. Cargo config in `Cargo.toml` and toolchain pin in `rust-toolchain.toml`.
- Python: code lives in `pygeodist/src/geodist`; tests in `pygeodist/tests`; developer tooling in `pygeodist/devtools`; packaging config in `pygeodist/pyproject.toml`.

## Build, Test, and Development Commands

- Python setup: `cd pygeodist && uv sync --all-extras` (installs deps into uv-managed venv).
- Python environment: run all Python commands via `uv` (e.g., `uv run ...`, `uv pip ...`) so the managed virtual environment is always active and consistent.
- Python lint/format/typecheck: `cd pygeodist && make lint` (codespell, ruff check+format, mypy).
- Python tests: `cd pygeodist && make test` (pytest over `src` and `tests`).
- Python build: `cd pygeodist && make build` (wheel + sdist).
- Rust run: `cd geodist-rs && cargo run`.
- Rust tests (when added): `cd geodist-rs && cargo test`.

## Coding Style & Naming

- Python: Ruff formatting/linting enforced; follow PEP 8, type-first design (`pyproject.toml` sets `strict = true` for mypy). Use snake_case for functions/vars, PascalCase for classes, module-level constants in SCREAMING_SNAKE_CASE.
- Module layout: keep Python public API definitions at the top of each `.py` file and place `_private` helpers after them so important entry points stay discoverable.
- Keep the PyO3 extension stub in sync: update `pygeodist/src/geodist/_geodist_rs.pyi` whenever the Rust-exposed API changes so typings mirror the compiled bindings.
- Rust: Standard Rustfmt defaults; favor small modules and explicit imports. Use snake_case for fns/vars, CamelCase for types/traits.
- Geospatial naming: use ISO-backed abbreviations for recurring concepts—`lat`/`lon` for latitude/longitude (ISO 6709) and `_deg` suffix for degree-valued angles (ISO 80000-3); prefer `_rad` when explicitly radians to avoid ambiguity. Keep these abbreviations consistent across code, tests, and docs.

## Documentation

- Keep docs concise and imperative. Avoid trivial docstrings/comments, but document public APIs and non-obvious helpers once they stabilize.
- For attributes (including module constants), annotate with Sphinx-style comments using `#: ...` so generated docs pick them up.
- Rust: Prefer RustDoc on public items and unsafe blocks; list what arguments mean, return values, panics/errors, and any invariants or safety preconditions. Include `# Safety` when relevant.
- Python: Use docstrings on public functions/classes/methods when behavior or contracts are non-trivial. Cover parameters (units/valid ranges), return values, raised errors, and important side effects. Skip docstrings for obvious data holders or passthroughs.

## Specs

- Author new specs under `docs/specs/proposed/` by copying `docs/specs/_TEMPLATE.md` and keeping its section headings/backlog table structure intact.
- Maintain the subfolders `proposed/`, `accepted/`, and `obsolete/`; move specs between them as status changes in the same PR that updates the document.

## Testing Guidelines

- Python: Pytest; place new tests under `pygeodist/tests` and mirror package layout (e.g., `tests/test_geom.py` for `src/geodist/geom.py`). Name tests `test_*` and use descriptive asserts. Add regression tests for bug fixes.
- Rust: Prefer unit tests co-located with modules and integration tests under `geodist-rs/tests` as functionality grows. Keep deterministic fixtures and avoid heavy external data.
- Aim for meaningful coverage around geometric kernels and witness-point reporting when implemented.

## Commit & Pull Request Practices

- Commit messages are short, imperative leading verbs (e.g., "Add formatter configuration"). Group related edits; avoid multi-topic commits.
- Pull requests should describe the change, note API impacts, and link issues if relevant. Include how to verify (commands run, screenshots for debug output where useful). Keep PRs small and focused on one feature or fix.

## Security & Configuration Tips

- No secrets should ever be committed; prefer env vars for credentials in future data-dependent benchmarks.
- Respect pinned toolchains: Python 3.10+ (per `pyproject.toml`) and Rust toolchain in `rust-toolchain.toml`. If tooling versions change, update pins and mention in PR notes.

## CI and Automation

- Python snippets used in GitHub Actions must live as standalone scripts under `.github/scripts/` instead of being embedded inline in workflow YAML.
