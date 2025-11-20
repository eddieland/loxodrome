# MVP Rust Distance & Hausdorff Kernel

**Status:** ✅ Done (MVP kernel delivered; future tuning/expansion lives in follow-on specs)

## Purpose

- Define a lean first pass of the Rust kernel focused on point distances and Hausdorff distance for point sets/paths.
- Keep the shape compatible with future accuracy upgrades, performance tuning, and Python bindings without locking APIs prematurely.
- Coordinate with the PyO3 bootstrap (`2025-11-19_pyo3-integration-plan.md`) so the kernel can be exported as a minimal wheel for early smoke testing.

## Guiding Constraints

- Favor clarity over micro-optimizations; correctness and determinism are the initial priorities.
- Keep the public surface small: single entry points for point-to-point, batch distance calculation, and Hausdorff over point sets, returning `Result` for fallible paths.
- Avoid pinning to a specific ellipsoid implementation yet; start with a sane default (WGS84) and leave room for injectable models later.
- Design for easy FFI/PyO3 exposure (simple structs/enums, `#[repr(C)]` when needed) so `pygeodist` can bind without churn; keep bindings feature-gated to avoid burdening the core.

## Target Project Structure (Rust Crate)

```plaintext
geodist-rs/
├── Cargo.toml
├── benches/                  # Criterion benches gated behind `bench` feature
│   └── distance.rs
├── src/
│   ├── lib.rs                # Public surface: re-exports, feature flags, error/types module wiring
│   ├── main.rs               # Thin CLI wrapper; uses lib API, no logic lives here
│   ├── types.rs              # Point, Distance, Error enums/structs; validation helpers
│   ├── distance.rs           # Great-circle implementations and batch helpers
│   ├── hausdorff.rs          # Directed/symmetric Hausdorff built on distance kernels
│   └── algorithms/           # Pluggable strategies (spherical default, future ellipsoid/planar)
│       ├── mod.rs
│       └── spherical.rs
├── tests/                    # Integration tests mirroring public API (distance, hausdorff, batches)
└── rust-toolchain.toml
```

- Keep the entry points discoverable in `lib.rs`; modules stay small and problem-focused.
- New algorithms land under `algorithms/` and are selected via features; API remains stable.
- Benchmarks and tests must use the library API to prevent drift between CLI and bindings.
- The structure is a target, not a straitjacket: if modules grow unwieldy or a layout stops making sense, reorganize with clear rationale and update this spec to match the implemented reality.
- Naming: the public point-to-point API is `geodesic_distance` (explicit to avoid confusion with Hausdorff or batch helpers); future algorithm-specific variants can follow the same pattern (e.g., `vincenty_distance`).

## Target Capabilities

1. Compute great-circle distance between two latitude/longitude pairs (degrees in, meters out) using a baseline spherical model (WGS84 radius).
2. Batch API that accepts slices/iterators for many-to-many or many-to-one use cases with predictable output ordering.
3. Compute Hausdorff distance between two point sets (directed and symmetric), reusing the distance kernel.
4. Configurable math backend for future swap-ins (e.g., Vincenty, Karney, planar fast paths) while keeping call sites stable.

## Subagent Execution Plan

### Task Backlog

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Define core types (`Point`, `Distance`, error enum) and input validation | Types live in `geodist-rs/src/types.rs`, re-exported from `lib.rs`; invalid inputs return errors; degrees documented | Keep types FFI-friendly; include doc comments | ✅ Done |
| P0 | Implement baseline great-circle `geodesic_distance` and targeted unit tests | Function `geodesic_distance(p1, p2)` returns meters; unit tests cover typical and polar/antipodal cases | Use `f64`; deterministic tolerances in tests | ✅ Done |
| P0 | Implement Hausdorff distance (directed and symmetric) over point slices | Functions `hausdorff(a, b)` and `hausdorff_directed(a, b)` reuse distance kernel; tests cover small sets and edge cases | Empty sets return `GeodistError::EmptyPointSet`; duplicates permitted | ✅ Done |
| P0 | Add batch helper (`geodesic_distances`) and tests | Accepts slice of point pairs; returns `Vec<f64>` or error | Prefer iterator-based internal impl to share logic | ✅ Done |
| P1 | Minimal Python binding surface | Follow `2025-11-19_pyo3-integration-plan.md`: add feature-gated PyO3 module exporting a trivial constant for wheel smoke tests, then expand toward distance/Hausdorff once stable | Keep module named `geodist._geodist_rs` (or equivalent) to avoid namespace clutter; PyO3 remains optional | ✅ Done |
| P1 | Benchmark harness stub | Add Criterion (or feature-gated) bench for distance and Hausdorff | Capture baseline numbers for future optimization | ✅ Done |
| P2 | Pluggable algorithm abstraction | Trait for algorithm strategy; spherical great-circle as default impl; Hausdorff accepts strategy | Enables drop-in higher-accuracy algorithms later | ✅ Done |
| P1 | Spatial index acceleration | Make `rstar`-backed nearest-neighbor search the default fast path for Hausdorff on large sets; keep a pure O(n*m) fallback for tiny inputs/tests | `rstar` becomes a baseline dependency; document when we fall back to the naive path | ✅ Done |
| P3 | Extended geodesic options | Optional ellipsoid selection, bearing output, and filtered/Hausdorff variants (e.g., clipped by bbox) | Only wire shapes; implementation can follow later | ✅ Done |

### Risks & Mitigations

- **Risk:** Numerical instability at extreme coordinates. **Mitigation:** Include edge-case tests (poles, antipodal), allow algorithm swap later.
- **Risk:** FFI surface churn when Python bindings land. **Mitigation:** Keep argument/return types minimal and C-friendly; gate breaking changes behind feature flags once stable.
- **Risk:** Performance regressions once higher-accuracy methods are added. **Mitigation:** Add a benchmark harness early and document tolerances for CI thresholds.
- **Risk:** Hausdorff over large point sets can be O(n*m). **Mitigation:** Document complexity; add early pruning options (bbox, thresholds) in future iterations.
- **Risk:** Introducing spatial index deps increases build/download surface. **Mitigation:** Accept `rstar` as a default dep; keep a documented fallback path and bench both to justify the added crate.
- **Risk:** PyO3 feature could bloat builds or change linker expectations. **Mitigation:** Keep `python` feature off by default, mirror steps in the PyO3 plan, and ensure the core crate still builds as `rlib` without PyO3 present.

### Open Questions

- Should angles be accepted in degrees only, or allow radians behind a feature flag?
- Do we need a soft-dependency on `geo`/`proj` crates for validation, or keep zero-deps initially?
- How much error tolerance is acceptable for the baseline spherical model in initial tests (e.g., 1e-4 relative vs. fixed meters)?
- **Heuristic:** Default to the naive O(n*m) path when `min(|A|, |B|) < 32` or `|A| * |B| <= 4_000` (keeps tiny cases fast and cheap). Build an R*-tree over the larger set otherwise; rebalance if we later expose batch updates. Validate/tune this threshold with Criterion benches (structured like Shapely/GEOS patterns of switching once index build amortizes ~log n query cost around a few dozen nodes).
- Should PyO3 bindings live in-core behind a feature or move to a dedicated bindings crate once APIs solidify?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _Extended geodesic options_
- **Overall spec state:** Done; follow-on algorithm/accuracy upgrades will be tracked separately.

## Lessons Learned (ongoing)

- Criterion is feature-gated to keep the default crate dependency-free.
- PyO3 module compiles cleanly behind the `python` feature when using the 0.22 `Bound<PyModule>` signature.
- Strategy trait keeps the public API stable while letting Hausdorff and batch helpers swap algorithms without call-site churn.
- Hausdorff uses an `rstar` nearest-neighbor path for larger sets and falls back to the O(n*m) loop for tiny inputs to avoid index build overhead.
- Mean-radius ellipsoid support keeps spherical math stable while allowing bearings and clipped Hausdorff variants to share the existing kernel.
