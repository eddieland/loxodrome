# Geometric Golden-Value Testing

## Purpose

- Codify a reusable suite of geometric “golden” values so distance, bearing, and Hausdorff outputs stay anchored to authoritative references.
- Catch regressions when kernels change by asserting against stable truths (e.g., WGS84 polar/meridional arcs, antipodal behavior) with tight tolerances and reproducible fixtures.
- Clarify how Rust and Python layers share the same verification data to prevent drift between bindings.

## Guiding Constraints

- Golden cases must cite a trusted source or derivation (Karney/GeographicLib, IERS WGS84 constants, analytic great-circle identities) and note expected tolerances.
- Keep tests deterministic: fixed seeds for any randomized case generation and pinned input ordering for Hausdorff paths.
- Cover both degree and radian entry points while enforcing consistent `lat`/`lon` semantics and `_deg` suffix usage.
- Prefer minimal external data; embed small fixtures alongside code but document how to regenerate heavier sets if needed.
- Test runtime should stay under seconds in CI; favor small, representative cases over exhaustive grids.

## Target Capabilities

1. Shared golden catalog of point pairs, paths, and expected outputs (distance meters, bearings degrees, witness points when added) consumed by both Rust and Python tests.
2. Deterministic tolerance policy per metric (absolute/relative errors) with rationale tied to floating-point stability and kernel formulas.
3. Regression harness for edge geometries: antipodal pairs, pole crossings, equator-aligned meridians, tiny separations, and altitude-influenced 3D chords.
4. Golden Hausdorff scenarios for symmetric/directed variants and bounding-box clipping to validate search path selection.
5. Documentation for regenerating or extending goldens (scripts + references) without manual guesswork.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Inventory canonical sources and derive golden cases for great-circle, ellipsoidal, and bearing outputs | Cases listed with source citations, expected values, tolerances, and unit conventions; checked into repo as structured fixtures | Seed with Karney/GeographicLib reference points, WGS84 quarter/half meridians, and antipodal pairs | 📝 |
| P0 | Build shared golden loader usable from Rust and Python test suites | Fixtures consumable without duplication (e.g., TOML/JSON + helper modules); CI-ready with deterministic ordering | Avoid serde drift; ensure Python typing lines up with fixture schema | 📝 |
| P0 | Add regression tests for edge geometries and failure modes | Tests assert against goldens for poles, equator, convergence near zero-distance, and invalid inputs | Include bearing wrap-around assertions (0/360) and NaN rejection | 📝 |
| P1 | Establish Hausdorff goldens (directed, symmetric, clipped) covering R*-tree vs fallback paths | Fixture paths with expected meters and witness indices (when available); tests validate both code paths | Include degenerate single-point sets and bbox exclusion cases | 📝 |
| P1 | Define tolerance policy doc + helpers | Centralized helpers for absolute/relative error thresholds with references to floating error analysis | Document why each tolerance exists; ensure reuse across tests | 📝 |
| P2 | Add reproducible script to regenerate goldens from reference solver(s) | Scripted generation with pinned versions and explanation of parameters; outputs overwrite fixtures deterministically | Prefer GeographicLib CLI or Karney test sets; guard against license issues | 📝 |
| P3 | Integrate property-based spot checks aligned to goldens | Limited proptest/pytest-checks seeded for CI with narrow bounds that reinforce golden expectations | Focus on invariants (symmetry, triangle inequality where applicable) around golden seeds | 📝 |

### Risks & Mitigations

- **Risk:** Diverging goldens between Rust and Python due to independent fixture copies. **Mitigation:** Single source fixture files plus thin loaders in each language.
- **Risk:** Overly tight tolerances causing flaky CI on some architectures. **Mitigation:** Document expected numerical stability; use architecture-aware thresholds where justified and log actual error margins.
- **Risk:** Reference solver or constants change upstream. **Mitigation:** Pin tool versions in regen scripts and record constant values (WGS84 radii/flattening) inside fixtures.
- **Risk:** Hausdorff goldens miss tree/fallback boundary conditions. **Mitigation:** Include minimal-size datasets that straddle R\*-tree switching thresholds and assert which path was taken when observable.

### Open Questions

- Which external reference solver(s) do we standardize on (GeographicLib CLI, PROJ, custom analytic cases)?
- How should we document expected witness points once kernels expose them (indices vs coordinates)?
- Do we want build-time gating that fails if generated goldens differ from committed fixtures?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _None yet._
- **Next up:** _Inventory canonical sources and derive initial golden cases._

## Lessons Learned (ongoing)

- _(To be filled as work progresses.)_
