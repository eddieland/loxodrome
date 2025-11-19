# Repository Guidelines

## Project Structure & Modules

- Root contains Rust crate `geodist-rs` (core kernels) and Python package `pygeodist` (bindings and tooling).
- Rust: see `geodist-rs/src`; binary entry in `src/main.rs` for now. Cargo config in `Cargo.toml` and toolchain pin in `rust-toolchain.toml`.
- Python: code lives in `pygeodist/src/geodist`; tests in `pygeodist/tests`; developer tooling in `pygeodist/devtools`; packaging config in `pygeodist/pyproject.toml`.

## Build, Test, and Development Commands

- Python setup: `cd pygeodist && uv sync --all-extras` (installs deps into uv-managed venv).
- Python lint/format/typecheck: `cd pygeodist && make lint` (codespell, ruff check+format, mypy).
- Python tests: `cd pygeodist && make test` (pytest over `src` and `tests`).
- Python build: `cd pygeodist && make build` (wheel + sdist).
- Rust run: `cd geodist-rs && cargo run`.
- Rust tests (when added): `cd geodist-rs && cargo test`.

## Coding Style & Naming

- Python: Ruff formatting/linting enforced; follow PEP 8, type-first design (`pyproject.toml` sets `strict = true` for mypy). Use snake_case for functions/vars, PascalCase for classes, module-level constants in SCREAMING_SNAKE_CASE.
- Rust: Standard Rustfmt defaults; favor small modules and explicit imports. Use snake_case for fns/vars, CamelCase for types/traits.
- Docs and examples: concise, imperative. Keep public APIs documented once they stabilize.

## Specs

- Author new specs under `docs/specs` by copying `docs/specs/_TEMPLATE.md` and keeping its section headings/backlog table structure intact.

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
