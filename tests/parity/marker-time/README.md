# Marker Time live Native source-kernel authority

Only genuine inputs live here: hand-written, generated-by-free `path.gcode` and
unchanged `tagged-overrides.json`. Native JSON, CSV, TSV projections and run logs
are generated into unique **external** artifact directories, never read from Git.
Historical outputs remain in git history, not as fallback oracle files.

Approved boundary: `docs/architecture/web-marker-time.md`,
`docs/architecture/web-colorchange.md` (tagged marker Time), and
`docs/architecture/marker-time-runtime-native-migration.md` (runtime authority).
Pinned upstream: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This is a **Native source-kernel diagnostic, not full Native Viewer/GL execution
or full-slicer G-code/M73 parity**. It executes real `GCodeProcessor` and
`libvgcode::convert` (`LibVGCodeWrapper.cpp`), the source-extracted inclusive
`ViewerImpl::get_estimated_time_at` ordered-f32 kernel, and unchanged
`Utils.hpp::get_time_dhms`. `tools/chain-probe/marker-time/` retains source excerpts,
adaptation diffs and extraction hashes. No Rust/JS-generated expected timing/text.

## Provision outside test deadlines

Use the pinned upstream source and nixpkgs paths, without changing Native patches:

```sh
export ORCA_PROCGCODE_SOURCE=${TMP_ROOT}/ares-orca-procgcode-source
export ORCA_PROCGCODE_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source
node tools/chain-probe/extract-marker-time.mjs --check
raw=$(nix build --impure --cores 8 --no-link --print-out-paths -f tools/chain-probe/marker-time.nix)
stable=$(nix build --impure --cores 8 --no-link --print-out-paths -f tools/chain-probe/marker-time-stable-options.nix)
export ARES_MARKER_TIME_BIN="$raw/bin/orca-slicer"
export ARES_MARKER_TIME_STABLE_BIN="$stable/bin/orca-slicer"
export ARES_REQUIRE_ORCA=1
```

Native/Node missing: optional mode visibly **SKIPPED**, required mode **FAIL**, even
in CI. Configured empty/missing/non-executable Native, Native nonzero/timeout,
invalid schema or missing output always **FAIL**, never a frozen fallback.

## Public-seam tests and repeatable artifacts

```sh
cargo nextest run -p ares-vgcode -p ares-wasm \
  --test marker_time --test marker_time_tagged --success-output immediate
# Numeric formatter seam; use the desktop project in the ordinary web config.
(cd web && node node_modules/@playwright/test/cli.js test markerTimeFormat.spec.ts --project desktop)
node --test tools/chain-probe/marker-time-runner.test.mjs
```

Each public-seam invocation prints its unique external artifact path. Rust support
`tests/support/marker_time.rs` uses shared Native availability policy and calls the
Node producer without depending on ares-cli. Browser numeric tests call that same
producer through `web/e2e/markerTimeNative.ts`; global browser oracle policy is
unchanged. Production core/vgcode/WASM/display code remains I/O-free and unchanged.

For a standalone raw capture, an external destination is mandatory:

```sh
artifact=$(mktemp -d /tmp/marker-time-raw.XXXXXX)
node tools/chain-probe/marker-time-fixtures.mjs "$artifact"
```

The raw capture contains 71 converted vertices, six synthetic numeric records,
48 C++ formatter cases, processor moves, mechanical TSVs, executable/source/input/
output SHA receipts, argv, stdout/stderr and lifecycle status. Rust still checks all
original exact bits, repeated IDs, filter independence, reverse seeks, reload and
seam-property-window assertions. Formatter still compares every C++ text record.

Tagged tests first slice the unchanged `orca_cli_ender3.3mf`/overrides through
`slice_project_plate_processed`, transfer public WASM buffers, and capture the
actual G-code externally. Only the existing generated-by identity adaptation is
applied to the Native input. At 345efec64 the raw producer is byte-identical to
the stable one on this fixture, so the raw/stable pair and the bounded-impact
proof were retired: `marker-time-stable.mjs` runs the single qualified beta
producer within its 300s deadline. Every stable identity, both duration modes
and Normal elapsed bits are still asserted.
To repeat the single-producer capture using that test's retained input:

```sh
stable=$(mktemp -d /tmp/marker-time-stable.XXXXXX)
node tools/chain-probe/marker-time-stable.mjs <printed-tagged-artifact>/oracle-input.gcode "$stable"
```

The producer rejects repository destinations and stale Native output admission.
`marker-time-runner.mjs` owns deadline cleanup, including descendants, and retains
failed/nonzero/timed-out receipts. Its short launcher controls are protocol tests,
not Native numerical evidence. Never tune counts, epsilon, fixture data or source
arithmetic to accommodate a failed live comparison.
