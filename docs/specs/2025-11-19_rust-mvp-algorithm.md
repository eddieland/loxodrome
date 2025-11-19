# MVP Rust Distance & Hausdorff Kernel

## Purpose

- Define a lean first pass of the Rust kernel focused on point distances and Hausdorff distance for point sets/paths.
- Keep the shape compatible with future accuracy upgrades, performance tuning, and Python bindings without locking APIs prematurely.

## Guiding Constraints

- Favor clarity over micro-optimizations; correctness and determinism are the initial priorities.
- Keep the public surface small: single entry points for point-to-point, batch distance calculation, and Hausdorff over point sets, returning `Result` for fallible paths.
- Avoid pinning to a specific ellipsoid implementation yet; start with a sane default (WGS84) and leave room for injectable models later.
- Design for easy FFI exposure (simple structs/enums, `#[repr(C)]` when needed) so `pygeodist` can bind without churn.

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
| P0 | Define core types (`Point`, `Distance`, error enum) and input validation | Types live in `geodist-rs/src/types.rs`, re-exported from `lib.rs`; invalid inputs return errors; degrees documented | Keep types FFI-friendly; include doc comments | Done |
| P0 | Implement baseline great-circle `geodesic_distance` and targeted unit tests | Function `geodesic_distance(p1, p2)` returns meters; unit tests cover typical and polar/antipodal cases | Use `f64`; deterministic tolerances in tests | Done |
| P0 | Implement Hausdorff distance (directed and symmetric) over point slices | Functions `hausdorff(a, b)` and `hausdorff_directed(a, b)` reuse distance kernel; tests cover small sets and edge cases | Empty sets return `GeodistError::EmptyPointSet`; duplicates permitted | Done |
| P0 | Add batch helper (`geodesic_distances`) and tests | Accepts slice of point pairs; returns `Vec<f64>` or error | Prefer iterator-based internal impl to share logic | Done |
| P1 | Minimal Python binding surface | Expose distance and Hausdorff functions via `pyo3` or `ffi` scaffold with smoke tests in `pygeodist/tests` | Aim for parity with Rust API naming; skip perf tuning | |
| P1 | Benchmark harness stub | Add Criterion (or feature-gated) bench for distance and Hausdorff | Capture baseline numbers for future optimization | |
| P2 | Pluggable algorithm abstraction | Trait for algorithm strategy; spherical great-circle as default impl; Hausdorff accepts strategy | Enables drop-in higher-accuracy algorithms later | |
| P2 | Optional spatial index acceleration | Prototype `rstar` (or similar) backed nearest-neighbor search to speed Hausdorff on large sets | Keep behind feature flag to preserve zero-dep core | |
| P3 | Extended geodesic options | Optional ellipsoid selection, bearing output, and filtered/Hausdorff variants (e.g., clipped by bbox) | Only wire shapes; implementation can follow later | |

### Risks & Mitigations

- **Risk:** Numerical instability at extreme coordinates. **Mitigation:** Include edge-case tests (poles, antipodal), allow algorithm swap later.
- **Risk:** FFI surface churn when Python bindings land. **Mitigation:** Keep argument/return types minimal and C-friendly; gate breaking changes behind feature flags once stable.
- **Risk:** Performance regressions once higher-accuracy methods are added. **Mitigation:** Add a benchmark harness early and document tolerances for CI thresholds.
- **Risk:** Hausdorff over large point sets can be O(n*m). **Mitigation:** Document complexity; add early pruning options (bbox, thresholds) in future iterations.
- **Risk:** Introducing spatial index deps increases build/download surface. **Mitigation:** Ship zero-dep core; gate `rstar` under a feature with clear benchmarks.

### Open Questions

- Should angles be accepted in degrees only, or allow radians behind a feature flag?
- Do we need a soft-dependency on `geo`/`proj` crates for validation, or keep zero-deps initially?
- How much error tolerance is acceptable for the baseline spherical model in initial tests (e.g., 1e-4 relative vs. fixed meters)?
- Should spatial indexing (`rstar`) ship as an optional feature now, or defer until we profile typical data sizes?

## Status Tracking (to be updated by subagent)

- **Current focus:** _Minimal Python binding surface_
- **Latest completed task:** _Add batch helper_
- **Next up:** _Benchmark harness stub_

## Lessons Learned (ongoing)

- _TBD_
