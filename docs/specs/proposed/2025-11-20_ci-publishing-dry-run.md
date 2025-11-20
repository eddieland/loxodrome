# CI Publishing Pipeline with Dry-Run Guard

## Purpose

- Stand up a CI-driven release path that builds the Rust crate and Python wheels from GitHub Actions while keeping releases safe during the private-repo phase.
- Ensure a clean handoff to real publishing (crates.io and PyPI) once the repo is public by reusing the same workflows with a small switch and secrets in place.

## Guiding Constraints

- Default to dry-run: build all artifacts and run `cargo publish --dry-run` / `maturin build` without uploading until explicitly flipped.
- Respect pinned toolchains and existing Make targets; avoid ad-hoc scripts that diverge from `pygeodist`/`geodist-rs` configs.
- GitHub Actions only; no self-hosted runners required. Matrix should cover target wheels we plan to ship but must remain time-bounded.
- Secrets-free dry run: workflows must succeed without registry tokens; uploads only happen when toggled and tokens are provided.
- Releases triggered from signed/tagged versions; no branch publishes.
- Registry prerequisites: crates.io package name reserved/linked to the maintainer account; PyPI project pre-created or test.pypi used for rehearsal; prefer GitHub Actions trusted publishing (OIDC) so no long-lived secrets are needed.

## Target Capabilities

1. Tag-driven workflow that builds and validates the Rust crate and all target Python wheels across OS/arch matrices.
2. Dry-run mode that archives build artifacts as CI outputs without hitting crates.io or PyPI.
3. Flip-to-live mechanism (env/flag) plus secrets wiring that turns on uploads with minimal changes once the repo is public.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Author GitHub Actions workflow for release tags | Workflow triggers on semantic tags, sets dry-run by default, and calls shared build steps | Keep Rust/Python steps reusable between dry-run and live modes | ✅ |
| P0 | Implement Rust crate packaging step | Runs `cargo publish --dry-run` (or `cargo package`) and uploads crate tarball as artifact | Ensure cargo uses workspace toolchain; fail on warnings | ✅ |
| P0 | Implement Python wheel build matrix | Uses `maturin build` to produce manylinux/macos/windows (x86_64 + aarch64 where supported) wheels and archives them | Reuse `uv sync` for deps; cache maturin/build outputs | ✅ |
| P0 | Add validation gates | Run lint/tests for Rust and Python before packaging; block release on failures | Align with `make lint`/`make test` targets where possible | ✅ |
| P1 | Define flip-to-live controls | Single env flag/input to toggle uploads; guard against uploads when secrets absent | Document required secrets and safety checks | ✅ |
| P1 | Wire upload steps (gated) | Add crates.io and PyPI publish steps behind flip flag and secrets | Use trusted publishing/oidc if feasible; otherwise token inputs | ✅ |
| P1 | Confirm/prepare registry namespaces | Reserve crates.io crate name and create PyPI (or test.pypi) project; document ownership and access | Default to GitHub Actions trusted publishing (OIDC); avoid long-lived tokens | 📝 |
| P2 | Document operator runbook | Update README/docs with how to tag, toggle, and verify artifacts | Include pointers to artifacts and rollback steps | ✅ |
| P3 | Post-publish notifications | Optional Slack/email release summary hooked to live publishes | Non-blocking; only after public release | 📝 |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Accidental live publish while private. **Mitigation:** Default to dry-run, require explicit input plus presence of tokens; add guard that fails if repo is private when live mode is requested.
- **Risk:** Wheel matrix runtime/cost too high. **Mitigation:** Start with minimal platform set (linux/macos/win x86_64) and expand once stable; cache builds; allow nightly/weekly dry-run instead of every push.
- **Risk:** Mismatched versions between crate and wheels. **Mitigation:** Single source of version (pyproject/Cargo) and a CI check ensuring tags match both manifests before packaging.
- **Risk:** Missing secrets/config when flipping live. **Mitigation:** Preflight checks for required secrets; short-circuit with actionable error messages.

### Open Questions

- Which exact wheel targets are mandatory for first release (universal2 macOS vs per-arch, musl support)?
- Can we rely solely on GitHub Actions trusted publishing (OIDC) for both PyPI and crates.io, or is any token fallback needed?
- Should dry-run artifacts be retained long-term (e.g., release assets) or pruned after inspection?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _Dry-run tag workflow, cross-platform wheel builds, and live upload guards/testing gates landed._
- **Next up:** _Confirm crates.io/PyPI namespace readiness and plan notification wiring for public releases._

## Lessons Learned (ongoing)

- Dry-run stays the safest default when registry tokens are absent; combining a repo-private guard with secret checks keeps accidental publishes blocked.
