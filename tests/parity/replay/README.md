# Replay inputs and runtime Native references

Replay behavioral tests slice these unchanged inputs with the current Native
OrcaRunner through its existing `slice_case` cache:

| Input | SHA-256 |
| --- | --- |
| `../orca_cli_ender3.3mf` | `7fb06c23baed9c8e06757cc0f7128c827e0b232eaa2655af687e4708014037c5` |
| `ender3-classic.3mf` | `abd8882f86e789f727fe362a11e3b57eb52e846c712b7a0b102b0a4fe3d05bc3` |

The classic input is the original ZIP with only the effective
`Metadata/project_settings.config` value `wall_generator` changed from
`arachne` to `classic` (JSON whitespace reserialized; all other entry bytes
preserved). These are labelled fixtures, not all-domain acceptance evidence.

Upstream source is OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`libslic3r/GCode.cpp:2030,2461` exports the complete stream, `:2576` emits the
producer/date header and `:5591-5643` serializes the full configuration.
Replay compares complete ordered bytes with only a validated generator/date
line normalized. Timing, M73, motion, whitespace, statistics and configuration
remain part of the comparison.

Both test fixture-copy sites use runtime Native bytes. Producer identity,
resources, command recipe, input SHA and retained raw outputs are bound by the
existing cache receipts outside Git. Required mode (`ARES_REQUIRE_ORCA=1`)
fails on missing dependencies before a cache hit; optional mode visibly SKIPS
Native-dependent tests before generating replay fixtures or comparison artifacts.
Transport error controls and the explicitly offline control remain independent
of live slicing. The divergence regression still mutates exactly one `M104 S`
to `M104 S9` in a temporary real Native stream, requires DIVERGENT and checks
retention of both complete streams.

The classic comparator's eleven categories and 33 mutation checks now use
runtime initialized Native in
`crates/ares-cli/tests/orca_parity/runner/stock_project_native_stream/classic.rs`.
The historical classic Native product is removed; no frozen fallback remains.
The CONFIG_BLOCK encode round trip slices `../orca_cli_ender3.3mf` with runtime Native
(`crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip.rs`).

The replay entrypoint itself consumes a supplied external layout: `case.3mf`
and `case/plate_1.gcode`. Set `ARES_PARITY_REPLAY` to that input directory and
`ARES_PARITY_ARTIFACT_ROOT` to a fresh absolute external directory, then run:

```sh
cargo nextest run -p ares-cli --test orca_parity \
  -E 'test(=replay::orca_parity_replay_sweep)'
```

Reports retain `replay-summary.json` and per-case `input.3mf`, `orca.gcode`,
`ares.gcode`, `manifest.json`, plus available failure diagnostics. Saved bytes
are bound by hashes and identify the executing Ares test binary. The legacy
replay manifest honestly records reference producer provenance as unknown;
a cache receipt does not alter that schema. Existing files are not overwritten.
Unset replay input is visibly offline, not evidence of a Native comparison.
See `docs/architecture/native-artifact-cleanup-replay-reference-spec.md` for
migration scope, assertion contracts and remaining deferred consumers.
