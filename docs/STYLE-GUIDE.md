# Geodist Style Guide

- Purpose: quick reference distilled from AGENTS.md, repo configs, and tool defaults.

## Formatting Defaults

- Line length 120; docstring code blocks target 88 characters.
- Python format via `ruff format` (Black rules): double quotes, spaces for indent, keep trailing commas, LF endings.
- Python lint via `ruff check --fix` with Google docstrings; run `make fmt` or `make lint` from `pygeodist`.
- Rust format via `cargo fmt`; Clippy fixes allowed in `make fmt` under `geodist-rs`.

## Python Style

- Target Python 3.10+; mypy runs in strict mode over `src` and `tests` (`make lint`).
- Naming: snake_case for functions/vars, PascalCase for classes, SCREAMING_SNAKE_CASE for module constants.
- Docstrings: add for public or non-obvious items; use Google style; skip trivial data holders/pass-throughs.
- Attributes (including module constants): document with Sphinx-style `#: ...` comments so autodoc captures them.
- Layout: put public API objects at the top of each `.py` file and follow with `_private` helpers to keep important entry points obvious.
- Types first: prefer explicit return/arg types; keep `pygeodist/src/geodist/_geodist_rs.pyi` in sync with Rust exports.
- Tests live in `pygeodist/tests` mirroring package layout; name `test_*`; favor descriptive asserts and regressions.
- Commands: `uv sync --all-extras` to set up; `make fmt`, `make lint`, `make test`, `make build` within `pygeodist`.

## Rust Style

- Toolchain pinned to nightly with rustfmt/clippy/rust-src/llvm-tools (see `geodist-rs/rust-toolchain.toml`).
- Naming: snake_case for functions/vars, CamelCase for types/traits; prefer small modules and explicit imports.
- Documentation: use RustDoc on public items and `unsafe` blocks; describe args, returns, panics/errors, invariants; include `# Safety` when needed.
- Commands: `make fmt` (rustfmt + clippy --fix), `make lint` (clippy -D warnings), `make test` (cargo nextest).

## Docs and Specs

- Keep prose concise and imperative; avoid boilerplate comments.
- New specs go under `docs/specs` by copying `_TEMPLATE.md`, keeping headings and backlog table intact.

## Git and PR Habits

- Commits: short, imperative messages; group related changes and avoid multi-topic commits.
- PRs: describe the change, note API impacts, link issues if any, and include how to verify (commands, outputs).

## Security and Config

- Respect pinned toolchains (Python 3.10+ per `pyproject.toml`, Rust per `rust-toolchain.toml`).
