# Merge-queue web build artifact reuse

## Decision

Use the successful pull-request build as a tree-addressed producer for the two
expensive WebAssembly outputs, then let the merge-queue `web-build` consumer
restore them only when the checked-out tree is identical. Keep the existing
merge-queue build and all browser checks as the consumer of those outputs.

The producer is a new pull-request path in the Tier 1 workflow. It runs the
existing build script with both WebAssembly outputs absent, records
`git rev-parse HEAD^{tree}`, and uploads only these generated directories:

- `web/packages/ares-wasm-js/src/wasm`
- `target/web-e2e/panic-probe`

The upload includes a manifest containing the tree SHA, toolchain pins, output
file paths, and SHA-256 values. The artifact name includes the tree SHA. The
consumer finds the successful producer run for the pull request listed by the
merge-group event, downloads the artifact by run ID, and verifies the manifest
and every file hash against the current tree before restoring either directory.
A missing run, missing artifact, tree mismatch, manifest mismatch, or hash
mismatch is a cache miss and takes the existing build path; it must not skip a
check or accept a non-identical artifact.

The held idea of populating a panic-probe cache from the main push is dropped.
The tree-addressed producer supplies the probe to the queue without adding a
cold probe build to the Pages deployment path. Existing content-addressed
caches remain useful for ordinary jobs, but are not used as proof that a
merge-group artifact is safe.

## Why the current path misses the target

The current complete-suite selection is fail-closed for every non-document
merge-group change (`scripts/ci/select-pr-checks.sh:4-11,106-123`). The
`web-build` job then installs the Rust and web toolchains, restores exact-input
WebAssembly caches, and runs the combined build (`.github/workflows/tier1.yml:230-272`).
Its script still builds the release host tools, runs the three WASM-producing
nextest suites, builds or restores shipped bindings and the panic probe, builds
the web packages, typechecks, and packs the consumer bundle
(`scripts/ci/run-web-build.sh:7-37`). The shipped build itself is a release
`wasm32-unknown-unknown` compile followed by `wasm-bindgen`
(`web/packages/ares-wasm-js/scripts/build-wasm.mjs:1-17`).

A prior pinned local run measured these paths with a fresh Cargo target and
provisioned tools:

| Path | Wall time | Directly measured phases |
| --- | ---: | --- |
| Original script, both outputs built | 1,001 s | — |
| Current script, both output caches absent | 562 s | shipped 339 s; panic probe 15 s |
| Current script, both output caches restored | 212 s | neither WebAssembly builder ran |
| Current script, restored outputs and warm Cargo target | 8 s | neither WebAssembly builder ran |

The cold run produced and verified 805/805 configuration keys and 17 required
bundle files. Its archive inventory, output checksums, and consumer checksums
matched the restored-output run. The 212-second path is therefore the upper
bound to beat for a fresh queue target after the expensive outputs are restored;
it already has 28 seconds of margin under the four-minute requirement. The
producer artifact removes the two longer compile phases while retaining the
host-tool build, metadata check, nextest suites, web build, typecheck, and
bundle packing in the queue consumer.

## Workflow shape

1. **Pull-request producer.** Add a producer job to the Tier 1 workflow for
   changes that select web work. It must check out the pull-request tree,
   install the pinned toolchain and dependencies, clear both output
   directories, and run the existing `scripts/ci/run-web-build.sh` with both
   output-hit variables false. It then records the tree manifest and uploads
   the two directories. A successful producer is the only source eligible for
   queue reuse. The pull-request path may continue to run the existing Pages
   checks independently; coupling the deployment workflow to this artifact is
   outside this slice.
2. **Merge-group consumer.** Before installing the Rust compiler or
   `wasm-bindgen`, query the workflow runs for the single same-repository pull
   request identified by the merge-group ref. Select only a completed,
   successful producer run. Download its tree-addressed artifact, verify the
   manifest against `HEAD^{tree}`, and restore the two directories. Export
   `WASM_CACHE_HIT=true` and `PANIC_PROBE_CACHE_HIT=true` only after both
   validations succeed.
3. **Fail-closed build.** If the producer cannot be found or any validation
   fails, leave both outputs absent and run the current compile path. Do not
   use prefix restores, a default-branch artifact, an artifact from another
   pull request, or a producer whose tree merely has matching WASM inputs.
4. **Unchanged consumer checks.** Run the existing build script after restore.
   It must still build the CLI and PrintConfigDef example, run its three
   nextest suites, verify metadata, build the web packages, typecheck, pack
   `target/ci-web-bundle`, and verify desktop shard balancing. Downstream jobs
   continue to restore that per-run bundle and execute the same streaming,
   desktop, package, and mobile suites (`.github/workflows/tier1.yml:274-361`).

The producer needs read access to pull-request and workflow-run metadata and
artifact download access. The workflow permissions must be explicit. The
lookup must reuse the existing single-PR and base/tree checks rather than
trusting a branch name: `scripts/ci/check-merge-group-reuse.sh:4-11,60-123`
shows the current fail-closed rules and exact tree comparison. Multiple pull
requests, a changed base, a rebased head, a fork, an API error, or an
in-progress producer must take the compile path.

## Included and deferred behavior

Included:

- A tree-addressed producer artifact for both expensive WebAssembly outputs.
- A successful-run lookup and manifest/hash validation in the queue consumer.
- Exact-hit variables passed to the existing script only after validation.
- Preservation of the current output bundle, browser consumers, checks,
  comparators, retry settings, and failure behavior.
- Removal of the main-push-only panic-probe producer from the design.

Deferred:

- General Rust compiler-cache infrastructure or a remote content-addressed
  object cache. The measured result is sufficient without introducing a new
  cache service.
- Parallel compilation or a new release profile. The current 212-second
  restored-output path meets the target and changing compilation settings would
  require a new output-equivalence campaign.
- Reworking the independent Pages pull-request build. Pages currently builds
  WebAssembly, the web app, package output, and its own checks
  (`.github/workflows/cloudflare-pages.yml:80-132`); this slice does not make
  deployment depend on a Tier 1 producer.

## Verification after approval

Run branch workflows only. First run the producer with an empty target and
empty output directories. Then run the merge-group consumer with a fresh target
and the producer artifact available, recording step timings and confirming the
build log contains no WebAssembly compiler or bindgen invocation. The consumer
run must complete in at most 240 seconds.

For both producer and consumer, compare the manifest hashes, final packed
bundle inventory, PrintConfigDef verification (805/805 keys), and the required
consumer files. Run the same web static checks and browser shards used by the
workflow. A negative fixture must change one tree byte and prove that the
consumer rejects the artifact and builds instead. A second negative fixture
must use a failed or incomplete producer run and prove that no output is
restored. Report misses as misses; never count a fallback build as an artifact
hit.
