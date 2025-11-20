# GitHub Actions CI Integration

**Status:** ✅ Done (CI workflows live; expand to additional platforms only if/when needed)

## Purpose

- Establish first-class CI coverage across platforms to keep Rust and Python builds healthy on every PR and push.
- Run full lint, tests, and benchmarking gates before merges; no release/publishing steps yet.

## Guiding Constraints

- Mirror local developer workflow: reuse `make lint`, `make test`, and existing dev tooling; avoid bespoke scripts.
- Support Python 3.10 and 3.12 on `ubuntu-latest` initially; macOS/Windows coverage is deferred but should be easy to re-enable.
- Keep PR CI fast and cancel superseded PR runs; avoid cancelling `main` branch builds.
- Prefer caching (uv/cargo/pip cache directories) to control runtime without sacrificing determinism.
- Benchmarks run on every PR (key regression detector) but stay non-blocking relative to lint/test signal.

## Target Capabilities

1. CI workflow dispatching on PRs and pushes to `main` with concurrency limits that cancel older PR runs.
2. Matrixed Python workflows covering 3.10 and 3.12 on Ubuntu (with room to reintroduce macOS/Windows later), executing lint and tests via `make` targets.
3. Benchmarks executed in CI (nightly or post-merge job) with results surfaced in logs and kept isolated from pass/fail gates unless configured.
4. Reasonable caching for Python and Rust dependencies to keep runtimes predictable across platforms.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Draft CI workflow skeleton for PRs/pushes with concurrency configured | Workflow triggers on PR and `main` pushes; concurrency group cancels in-flight PR runs but not `main`; lint/test steps stubbed | Use `concurrency` with branch check; ensure PR reuse | ✅ Done |
| P0 | Implement Python matrix job running `make lint` and `make test` for 3.10/3.12 on ubuntu | Jobs create uv environment, install deps, run lint and tests; all commands succeed on Ubuntu | macOS/Windows deferred until/if needed | ✅ Done |
| P0 | Validate Rust components build in CI (if needed by bindings) | Cargo build step passes on ubuntu | Skip other OS until needed; document rationale | ✅ Done |
| P1 | Add benchmark job separated from gating lint/test | Benchmark job runs on every PR on ubuntu, uploads logs/artifacts; does not block PR merge failures by default | PR-scope execution chosen to catch regressions early | ✅ Done |
| P1 | Add caching for Python (uv) and Cargo to reduce runtime | Cache keys include OS, Python version, lockfiles; cache restores validated | Verify cache paths per OS; avoid stale cache issues | ✅ Done |
| P2 | Add reporting/annotations for lint/test failures | Ruff/mypy/pytest outputs surfaced as annotations | Platform-neutral approach | ✅ Done |
| P3 | Explore optional code coverage publishing | Coverage artifacts kept for local download; no external services yet | Future enhancement | ✅ Done |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** uv or tool installation differences across macOS/Windows. **Mitigation:** use official install snippets per OS and pin versions; fall back to `python -m pip install uv` where needed.
- **Risk:** Matrix size slows feedback. **Mitigation:** prioritize ubuntu job completion first; enable caching and consider allowing mac/windows to run without blocking merge once signal is trusted.
- **Risk:** Benchmarks introduce flakiness/length. **Mitigation:** run benchmarks on schedule or post-merge and keep them non-blocking; capture artifacts for inspection.
- **Risk:** Cargo/uv cache poisoning after dependency bumps. **Mitigation:** include lockfile hashes in cache keys and add occasional cache bust controls.

### Open Questions

- Should PR merges wait for all matrix jobs or allow optional platforms to be non-blocking initially (once additional OSes are added)?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _CI workflow covering Python matrix lint/test, Rust build, benches, caching, annotations, and optional coverage artifacts._
- **Overall spec state:** Done; monitor runtime/cache hit rates and expand to macOS/Windows or gated coverage thresholds in follow-on revisions when justified.

## Lessons Learned (ongoing)

- uv and cargo caches keyed by lockfiles keep rebuilds tolerable while letting nightly toolchain installs remain simple; keep metadata hashes in keys to avoid stale caches.
