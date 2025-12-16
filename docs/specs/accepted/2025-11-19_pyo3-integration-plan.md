# PyO3 Integration Plan (Bootstrap)

**Status:** ✅ Done (bootstrap delivered; future API expansion deferred to follow-up spec)

## Purpose

- Introduce PyO3 as the Rust↔Python bridge for an initial binding.
- Ship a buildable constant export to validate wheel generation in CI and locally.
- Keep API design flexible for a follow-up spec once kernels settle.

## Guiding Constraints

- Keep `loxodrome-rs` as the single source of truth; gate PyO3 behind an optional `python` feature.
- Publish the Rust extension under the `loxodrome` namespace (e.g., `loxodrome._loxodrome_rs`) without polluting top-level APIs.
- Use maturin for Python packaging while staying compatible with the existing `uv` workflow and pinned toolchains.

## Target Capabilities

1. Feature-gated PyO3 module exporting a minimal surface (constant or stub) that builds with and without the `python` feature.
2. Maturin-backed Python build wiring that produces a wheel from `loxodrome-rs` via `loxodrome/pyproject.toml`.
3. Python package re-export that allows `import loxodrome` to surface the bound constant with a smoke test validating parity.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Rust bindings shell | Optional `python` feature adds PyO3 and exports `EARTH_RADIUS_METERS` via `loxodrome._loxodrome_rs`; `cargo check` passes with and without the feature | Already merged; keep module naming stable for downstream imports | ✅ Done |
| P0 | Build system wiring | `loxodrome/pyproject.toml` uses maturin with `manifest-path` pointing at `../loxodrome-rs/Cargo.toml` and enables the `python` feature; Make/uv targets documented | Align with `2025-11-19_rust-mvp-algorithm.md` references | ✅ Done |
| P1 | Python surface | `loxodrome/__init__.py` re-exports the bound constant; smoke test asserts import works and value matches Rust | Keep Python namespace minimal and stable | ✅ Done |
| P1 | Validation | `uv sync --all-extras --dev`, `maturin develop` with the `python` feature, and pytest run documented (and added to CI if feasible) | Workflow documented in README; manual run confirmed via uv + maturin develop + pytest | ✅ Done |
| P2 | Future API expansion | Follow-up spec to design kernel function exports, error mapping, and data model | Defer until kernels stabilize | ⏸️ Deferred |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Module naming drifts from `loxodrome._loxodrome_rs`. **Mitigation:** Keep the module path stable and update dependent docs/tests immediately if a rename is required.
- **Risk:** Maturin conflicts with `uv-dynamic-versioning` or pinned toolchains. **Mitigation:** Verify backend compatibility before switching and document any required pin updates.
- **Risk:** Need to split bindings into a dedicated crate later. **Mitigation:** Keep PyO3 behind a feature flag and design wiring so manifest paths are easy to relocate.

### Open Questions

- Should validation run in CI immediately or remain manual until more APIs land?
- Do we need a second extension module if multiple kernels ship, or can we keep a single shared module?
- What criteria trigger promoting the bootstrap surface into a fuller API spec?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** Validation of the end-to-end Python build/test workflow (`uv sync --all-extras --dev`, `uv run maturin develop`, `uv run pytest`).
- **Overall spec state:** Done; kernels and API growth shift to the follow-on expansion spec.

## Lessons Learned (ongoing)

- PyO3 0.22 requires using `Bound<PyModule>` in the module signature for `#[pymodule]`; the older `&PyModule` form no longer exposes `add`.
- Surface friendly import errors to remind developers to run `maturin develop` when the extension module is missing.
- Run `uv run maturin develop` before invoking pytest so the extension module is present; maturin picks up the `python` feature from `pyproject.toml`.
