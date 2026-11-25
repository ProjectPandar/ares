# CI and GitHub Merge Queue Operations

This document describes how Ares runs continuous integration on public
`ubuntu-latest` GitHub-hosted runners with GitHub Merge Queue and versioned
GHCR oracle container images.

## Workflow Layout

| Workflow | Triggers | Purpose |
|---|---|---|
| `.github/workflows/oracles-build.yml` | `workflow_dispatch`, `push: main` on oracle recipe/patch paths | Builds the 6 pinned OrcaSlicer v2.5.0 Nix oracle variants, stages their deduplicated runtime closure + read-only `source-r3` preset catalog + SHA-256 manifest into a `FROM scratch` OCI image, pushes to `ghcr.io/<owner>/<repo>/ci-oracles:<tag>`, and prunes older tags (retaining the latest 3). |
| `.github/workflows/tier1.yml` | `pull_request`, `merge_group` (`checks_requested`) | Runs fast checks (`format`, `native-nonparity`) on every PR, and adds sharded `parity-official` (8 shards), `parity-seeded` (8 shards), and sharded Playwright E2E (`web-streaming`, `web-desktop` x2, `web-packages-mobile`) on `merge_group` (or PRs labeled `run-full-parity`). |
| `.github/workflows/cloudflare-pages.yml` | `pull_request`, `merge_group`, `push: main` | Runs `pages-build` (build, lint, typecheck, Pages Playwright suite) on PRs and merge groups; deploys to Cloudflare Pages on `push: main` when `CLOUDFLARE_PAGES_ENABLED == 'true'`. |

Daily PR and `merge_group` jobs never install Nix and never upload GitHub
Artifacts. Intra-run transfer of `orca-parity.tar.zst` and `web-dist.tar.zst`
uses run-scoped `actions/cache/save@v4` + `actions/cache/restore@v4`.

## One-Time GitHub Repository Setup

### 1. Publish and Expose the Initial GHCR Oracle Image

1. Trigger **Actions -> Build CI Oracles -> Run workflow** on `main` (uses the
   built-in `GITHUB_TOKEN` with `packages: write`; no external secrets required).
2. Wait for `package-image` to finish and note the published digest printed in
   the log (`ghcr.io/<owner>/<repo>/ci-oracles:v2.5.0-<hash>@sha256:...`).
3. Open **Packages -> `ares/ci-oracles` -> Package settings**:
   - Set visibility to **Public** so fork PRs and merge groups can pull cleanly.
   - Confirm repository access grants **Write** to the repository Actions workflow.
4. Optionally pin `ORACLE_IMAGE_DIGEST=sha256:...` in `scripts/ci/oracle-image.env`
   for immutable digest pinning in addition to the deterministic recipe tag.

### 2. Configure the `main` Branch Ruleset and Merge Queue

Under **Settings -> Rules -> Rulesets -> New branch ruleset** targeting `main`:

- **Enforcement status:** Active
- **Linear history:** Enabled (`Require linear history`)
- **Require pull request before merging:** Enabled
- **Require status checks to pass before merging:** Enabled
  - Add ONLY the two stable aggregator checks:
    - `tier1-gate`
    - `pages-build`
  - Never add individual matrix shard names (such as `parity-official (1/8)`)
    so shard counts can change without editing branch protection.
- **Require merge queue:** Enabled
  - **Merge method:** Squash (or Rebase, matching linear history)
  - **Build concurrency:** `1` on the public 20-concurrent-job tier (8 official
    + 8 seeded + 4 web = 20 jobs per merge group); raise to `2`–`3` on higher
    concurrency tiers.
  - **Minimum entries to merge:** `1`
  - **Maximum entries to merge:** `3`–`5`
  - **Wait time for minimum entries:** `1`–`2` minutes
  - **Status check timeout:** `90` minutes initially (reducible to `45` minutes
    after preview rendering optimization lands).

## Updating Pinned Oracles

1. Edit the relevant recipe or patch under `tools/chain-probe/` or
   `scripts/ci/build-oracle.sh`.
2. `scripts/ci/oracle-image.sh tag` computes a new deterministic
   `v2.5.0-<12-hex>` tag automatically.
3. Run `Build CI Oracles` (or push the recipe update to `main`) so the new tag
   exists in GHCR before dependent PRs enter the merge queue.
4. `oracles-build.yml` automatically prunes untagged and older GHCR package
   versions beyond the configured retention count (`keep_versions`, default `3`).
   Never push `native-cache-v2` outputs to GHCR.

## Local Validation Helpers

```bash
# Check deterministic oracle image tag and reference
bash scripts/ci/oracle-image.sh tag
bash scripts/ci/oracle-image.sh ref

# Validate shell syntax across all CI scripts
bash -n scripts/ci/*.sh

# Smoke-test nextest archive and hash partitioning locally
cargo nextest archive -p ares-cli --test orca_parity --archive-file /path/to/orca-parity.tar.zst
cargo nextest list --archive-file /path/to/orca-parity.tar.zst --workspace-remap "$PWD" \
  --profile parity-official --partition hash:1/8
```
