# CI and GitHub Merge Queue Operations

This document describes how Ares runs continuous integration on public
`ubuntu-latest` GitHub-hosted runners with GitHub Merge Queue, diff-based PR
selection, strict latency budgets, and versioned GHCR container images.

## Workflow Layout

| Workflow | Triggers | Purpose |
|---|---|---|
| `.github/workflows/tier1.yml` | `pull_request`, `merge_group` (`checks_requested`), `push: main` | Runs `fast-checks` (merged small checks + strict ms budgets) alongside parallel `rust-wasm-check`, `rust-clippy`, and `rust-nonparity` on PRs using diff selection (`scripts/ci/select-pr-checks.sh`). On `merge_group` (or PRs labeled `run-full-parity`), bypasses selection and runs the complete suite: 8 `parity-official` shards + 8 `parity-seeded` shards + 4 timing-balanced `web-desktop` shards + `web-streaming` + `web-packages-mobile` with mathematical shard completeness proofs. |
| `.github/workflows/cloudflare-pages.yml` | `pull_request`, `merge_group`, `push: main` | Runs `pages-build` (reusing content-addressed `ares-wasm-pkg` cache on web-only PRs, skipping in ~12s on non-web PRs, always rebuilding fresh on `merge_group`). On `push: main`, `deploy` reuses the cached `web/app/dist` (~24s deploy). |
| `.github/workflows/oracles-build.yml` | `workflow_dispatch`, `push: main` on oracle recipe/patch paths | Builds the 6 pinned OrcaSlicer v2.5.0 Nix oracle variants (cached by deterministic recipe tag), stages their deduplicated runtime closure + preset catalog into `ghcr.io/<owner>/<repo>/ci-oracles:<tag>`, and retains the latest 3 versions. |
| `.github/workflows/toolchain-build.yml` | `workflow_dispatch`, `push: main` on `scripts/ci/toolchain-image.*` or `ci-toolchain.Dockerfile` | Builds and publishes SHA-256-verified `ghcr.io/<owner>/<repo>/ci-toolchain:base-<hash>` and `browsers-<hash>` images (Rust `1.99.0`, `cargo-nextest`, `wasm-bindgen`, pinned Bun canary, `hk`, Playwright). |

## Non-Negotiable Rules & Latency Budgets

1. **Complete suite before `main`**: `merge_group` always runs 100% of native, parity, and web E2E tests unconditionally. Diff selection (`scripts/ci/select-pr-checks.sh`) narrows PR feedback only.
2. **Never hide failures**: zero retries (`--retries 0`), no relaxed timeouts, no skipped flaky tests.
3. **Never reduce checks**: lint rules (`web/.oxlintrc.json`), formatters (`bun fmt`, `cargo fmt`), TS7 `bun check`, unit tests (`bun test --randomize`), and required status checks only stay or grow.
4. **Enforced budgets** (`scripts/ci/run-fast-checks.sh` fails any check exceeding its budget):
   - PR wait: `< 5s` (`cancel-in-progress: true` on `pull_request`)
   - `bun fmt --check`: `<= 1,000ms` (measured `~31ms`)
   - `bun lint --max-warnings=0`: `<= 1,000ms` (measured `~172ms`)
   - `bun check` (TypeScript 7): `<= 5,000ms` (measured `~1,540ms` incl. `.d.ts` + profile build)
   - `bun test --randomize`: `<= 5,000ms` (measured `~600ms`)
   - Local `hk` hooks (`hk.pkl`): `< 9s`

## Diff-Based PR Selection & Full-Rerun Triggers

`scripts/ci/select-pr-checks.sh` runs locally inside each parallel PR job (zero serial `plan` job wait). Touching any of the following **full-rerun triggers** forces every PR check to run:

- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.config/nextest.toml`, `.cargo/*`, `vendor/*`
- `web/package.json`, `web/bun.lock`, `web/tsconfig*.json`, `web/.oxlintrc.json`, `web/bunfig.toml`, `web/packages/ares-wasm-js/scripts/build-wasm.mjs`
- `.github/workflows/tier1.yml`, `.github/workflows/cloudflare-pages.yml`, `.github/actions/*`
- `scripts/ci/*`, `scripts/gen-configdef*`, `scripts/verify-configdef.mjs`
- Any uncategorized path (fail-closed default).

## Shard Balancing & Completeness Proofs

- **Web E2E (`scripts/ci/balance-web-shards.mjs`)**: partitions `web-desktop` specs into 4 disjoint `--workers 1` shards via LPT greedy bin-packing using recorded `main`/`merge_group` per-file timings, reports any spec file exceeding `20s` for splitting, and proves `union(shards) == 100% of project specs` in both `fast-checks` and `web-build`.
- **Parity (`scripts/ci/verify-parity-shards.sh`)**: proves in `build-parity` that the 8 `parity-official` and 8 `parity-seeded` `nextest` hash shards are disjoint and cover 100% of each profile's tests.

## Local Hooks (`hk`)

Install the enforced git hooks (`hk v2.14.0`):

```bash
hk install
hk validate
hk run pre-push
```

- `pre-commit`: runs `rustfmt --check`, `bun fmt --check`, `bun lint`, and `bash -n` on staged files (`< 1s`).
- `pre-push`: runs `cargo fmt --check`, `web fmt:check`, `web lint`, `balance-web-shards.mjs verify`, and `scripts/ci` checks (`< 9s`).

## One-Time GitHub Repository Setup

1. Ensure `ghcr.io/<owner>/<repo>/ci-oracles` and `ci-toolchain` packages have **Public** visibility in GitHub Packages settings for fork PRs.
2. Under **Settings -> Rules -> Rulesets** targeting `main`, enable:
   - `Require linear history`
   - `Require pull request before merging`
   - `Require status checks to pass before merging` with ONLY the two stable checks: `tier1-gate` and `pages-build`
   - `Require merge queue` (Rebase/Squash, build concurrency `1` on the 20-job public tier, min `1`, max `3`, timeout `90m`).
