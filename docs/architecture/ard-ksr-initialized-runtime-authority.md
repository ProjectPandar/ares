# KSR runtime Native authority

## Decision

The original `ksr_fdmtest_v4.gcode` capture's historical producer is not
reproducible. It is not the authoritative Native reference for Ares output.
The independently initialized producer recorded by ARD-0068/0069/0073/0074
and `initialized-reference.json` is authoritative for this KSR input.
It remains distinct from untouched official Native behavior.

Source boundary: OrcaSlicer `8500fcdcc`, `Print.cpp:2618-2638`
`Print::export_gcode`, `GCode::do_export`, and
`GCode.cpp:5591-5643` `GCode::append_full_config`. Rust consumer boundary:
CLI complete-output E2E and migrated CONFIG_BLOCK E2E. Core retains pure Rust
configuration checks and never runs Native. No production pipeline changes.

## Preflight conclusion

Two serial fresh captures use the unchanged KSR input, seed zero, single-core affinity,
and the historical initialized wrapper and ELF. Both executable SHA values
match their recorded identities, and both captures succeed.

- Raw initialized SHA differs solely because the validated generator timestamp
  changes. No other bytes change after the existing generator normalization.
- Normalized initialized whole-output bytes and hash reproduce exactly in both
  captures and agree with each other.
- Initialized CONFIG_BLOCK bytes and hash reproduce exactly in both captures:
  49065 bytes, with unchanged order, formatting, keys and values.
- Original capture's normalized whole-output and CONFIG_BLOCK hashes do not
  reproduce with that producer. First normalized difference is printing time
  on line 3; original block is 49005 bytes and first differs at block line 76.

Complete producer identity, commands, raw captures, all three hash classes and
first differences are retained outside Git in a hash audit and its capture
directories.
No Native output or generated metadata is added to the repository.

## Reconstructible producer source

The historical derivation's source was a content-addressed directory import,
not a fetch derivation. A fresh `git archive` of upstream revision
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, followed by the committed
`tools/orca-probes/ksr-initialized-source.patch`, reproduces its NAR SHA-256:
`1fbg964by6qdk22cx5c6anfr9jxkb6zgv0q0inhbpfgd87683phs`.
This verification reads Git and computes a hash only; it neither realizes
Native nor writes the reconstructed source into the system store.

The patch changes exactly `Feature/FuzzySkin/FuzzySkin.cpp`,
`Fill/FillConcentric.cpp`, `Fill/FillConcentricInternal.cpp` and `Print.hpp`.
The historical source does not include the current replay patch's `GCode.cpp`
placeholder-RNG change. Source import name
`orca-slicer-2.4.2-seed-replay-source` and the reproduced NAR hash identify
`/nix/store/hxvv0g1qlw9lwimj2km3ghxphrl9p4qf-orca-slicer-2.4.2-seed-replay-source`.

`tools/orca-probes/ksr-initialized.nix` requires this reconstructed source as
`ORCA_KSR_INITIALIZED_SOURCE`; it does not fall back to the historical store
source. It inherits the original patches, configure flags, dependencies and
wrapping from pinned nixpkgs `9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source`.
Instantiation with the reconstructed source reproduces exact derivation
`4n1w0b6jgnfcwb6m9jfssndribyyg3v6-orca-slicer-seed-replay-2.4.2.drv`,
whose package output is
`jqfsg42f6xzq3krnf5scm6w4g1z4kz4c-orca-slicer-seed-replay-2.4.2`.
The existing package remains protected by a garbage-collection root.
[tools/README](../../tools/README.md) records the reconstruction and
instantiate-only commands. This decision does not realize the package.

## OrcaSlicer 2.5.0 producer

The KSR consumers now compare against the 2.5.0 (`40ecc66f7`) seed-replay producer.
This replaces the 2.4.2 producer above. Upstream 2.5.0 already initializes the
fields that the 2.4.2 patch set:
- `Arachne::WallToolPathsParams` defaults `min_length_factor = 0.5f` and
  `is_top_or_bottom_layer = false` (`WallToolPaths.hpp:33,38`). The concentric fills
  therefore no longer need them set.
- `Print.hpp:742` declares `size_t m_id{0}`.

The 2.5.0 producer source is `40ecc66f7` plus the seed-only change. It patches only
`Feature/FuzzySkin/FuzzySkin.cpp` (the `ORCA_REPLAY_SEED` generator) and
`GCode.cpp` (the placeholder RNG seed). Its package output is
`6fp28jy4wfr292v1knh0wp8yp6a65v06-orca-slicer-seed-replay-250-2.5.0`, protected by a
garbage-collection root.

`producer.rs` binds the producer identity to the raw SHA-256 of both executables:
- wrapper `bin/orca-slicer`:
  `9d285c9995cdd0eacef354afb76853c81e3def3154b12e38a50238bf268004d6`;
- ELF `bin/.orca-slicer-wrapped`:
  `84a7780653be3b46fe909e43ae98b3576907c561e8ff337c208ff97c2b23a84a`.

The consumer names drop the version: `project_matches_initialized_orca_ordered_bytes`
and `browser::browser_output_matches_initialized_orca_ordered_bytes`. The comparator,
the input SHA and the cache binding do not change.

## Consumer contract

All Ares-versus-reference comparisons migrate to freshly generated initialized
Native output with the unchanged complete ordered generator-only comparator.
CONFIG_BLOCK comparisons use runtime block bytes directly, without introducing
a replacement fixed hash. Any Ares/runtime Native mismatch remains a real RED
and must be reported; no tolerance or output masking is permitted.

Assertions authenticating only a disappearing frozen file may disappear with
that file: historical raw/normalized hashes, frozen line count, producer/output
identity, and original CONFIG_BLOCK size/hash self-checks. The deletion
must list their exact original assertion text and individual deletion reasons.
Input SHA/package assertions and all Ares-versus-Native behavior assertions
remain. The original CONFIG_BLOCK hash is never rebound to the initialized
hash. Runtime reference integrity uses the existing producer-bound cache.

Native consumers follow shared required/optional availability policy before
cache use. Policy tests run in default environments using a temporary external
directory. Browser comparison retains actual WASM bytes, loses its old ignore
attribute, and receives explicit Native environment and artifact destination.

## Browser frozen-consumer policy RED

The old ignored browser consumer still compares the initialized frozen file
when every Native dependency is absent. A retained actual Chromium capture
from ARD-0082 reproduces this:
the child completes the whole ordered comparison, writes comparison artifacts,
and reports no SKIPPED. Its historical dated Ares generator line alone is
adapted to the current undated Ares header contract; body bytes are untouched.
This is a missing-dependency policy RED, not fresh browser GREEN evidence.
Audit and child logs are retained outside Git;
policy result is exit100.
The new browser consumer must generate Native through initialized
`OrcaRunner::slice_case`, visibly skip when dependencies are optional and
absent, fail when required, and require an actual browser capture once Native
is available. Fresh Chromium runs remain mandatory before final deletion.
