# Explicit up-to-date machine-file decoding

## Upstream boundary

This ports the direct-file decoding context used by
`OrcaSlicer.cpp::load_config_file` and its explicit up-to-date machine
consumer, not an independently designed preset pipeline. The source is
OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `OrcaSlicer.cpp:2205–2275` loads a `DynamicPrintConfig` with
  `load_from_json`, checks `from`, determines or probes `type`, resolves a
  preset only when necessary, and calls `normalize_fdm`.
- `resolve_preset`, `OrcaSlicer.cpp:2118–2155`, returns directly when `type`
  is explicit and `inherits` is absent or empty. Missing `type` instead
  invokes `resolve_preset_config_type`; nonempty `inherits` invokes
  `resolve_preset_config`. `system` uses the system resolver with source
  manifests permitted; `User`/`user` use the CLI preset bundle. Neither
  branch is a parse-error fallback.
- `OrcaSlicer.cpp:2539–2558`, under `--uptodate --uptodate-settings`, calls
  that loader, checks the machine name against the original system printer,
  and replaces the upward vector from `upward_compatible_machine`.
- `Config.cpp:978–1060` is the full-config JSON reader: known option
  strings/arrays are not restricted to printer-owned typed fields; other
  top-level value shapes do not supply option values.
- The separate `--load-settings` loop starts at `OrcaSlicer.cpp:2280` and
  uses the same loader. It is not the explicit up-to-date consumer.

## Observed premises

The checked-in `tests/parity/machine-full-resource/stock-machine-full.json`
is 16,306 bytes, SHA-256
`2a2e89e53063119290a5b8c185ea01f4e207044dc11f4e78f5a34cd22d162e4b`.
It declares `type: machine`, `from: system`, no `inherits`, and includes
`cooling_filter_enabled: "0"`. The untouched input archive is
`tests/parity/master-extruder-id/stock-right-only.3mf`, SHA-256
`69d425b79297129c1b3894ee106269d3b33b7f1badcbeed15598dfe774cff09a`.

Actual official and seeded Native runs cover both explicit-file consumers:
stock bytes and a single added `inherits: ""` field each return exit 0,
report code 0, and the exact ordered upward vector
`["Bambu Lab H2D Pro 0.4 nozzle"]`.

In the same external-file environment, adding
`inherits: "Bambu Lab H2D 0.2 nozzle"` returns exit 251 / code -5 with
`Preset was not found in the loaded bundle`; removing `type` returns
251 / -5 with `Preset type could not be resolved`. These are observations
of that resolver environment, not universal inherited or missing-type
contracts. Their bundle/manifest resolution is deferred.

The public Ares `UpToDateFiles` projection currently rejects stock and
empty-inherits bytes at `cooling_filter_enabled` through the strict fragment
parser. The raw CLI follows a different path: `--load-settings` alone with
only the machine file fails earlier because the selection lacks a process
and filaments. With the unchanged checked-in
`validation/load-settings-3mf/process.json` and
`compatible-filament-0.json` additionally supplied through their ordinary
CLI lists, both Native and Ares return exit 239 / code -17 for stock and
empty-inherits variants. Those Creality compatibility lists do not match
the H2D machine. That complete-list control records admission through to
the existing compatibility check; it is not a slicing-success case.

## Rust destination and non-negotiable decisions

The destination is `crates/ares-core/src/project/upward_machine.rs`'s
explicit-file branch and the crate-private decoders in
`crates/ares-core/src/profiles/fragment.rs`.

1. Select full-config decoding by validated machine metadata and
   absent/empty `inherits`, never by catching a strict-parser failure.
2. Reuse the canonical full-config conversion already used by original
   machine resources. It admits known non-printer options, uses canonical
   unknown-option handling, and omits unsupported top-level shapes. Do not
   add option definitions, ownership-list entries, coercions or defaults.
3. Keep public `ProfileFragment::from_json_bytes` strict. Preserve other
   file kinds, source validation, machine-name matching, process skipping,
   filament rejection, vector replacement, ordering and multiplicity.
4. Preserve original-resource behavior. Shared private conversion may be
   factored without changing its context selection or public behavior.
5. The byte-oriented upward projection remains a temporary compatibility
   shell around the cited upstream consumer, not a refresh controller.
   Core receives bytes and performs no filesystem, terminal, environment,
   native-viewer or platform-specific work.

## Included and deferred behavior

Included: explicit up-to-date machine-file admission for explicit machine
`type` with absent/empty `inherits`, and its ordered upward-vector result.
The existing complete-file-list raw CLI is a regression control only;
its canonical catalog-composition path is not changed.

Deferred: inherited and missing-type resolution, source manifests, bundle
or include orchestration, the full refresh controller and its CLI adapter,
partial CLI selections, process-refresh/name behavior beyond the existing
projection, and unrelated nozzle/grouping or slice-output differences.
No known Ares/Native mismatch becomes an expected test result.

## Public E2E exit criteria

- Before production changes, stock and empty-inherits public API cases must
  fail admission while their actual Native explicit-up-to-date runs succeed
  with the exact Pro upward vector, in both official and seeded modes.
- After the narrow fix, those public API results must equal the measured
  Native ordered vector. The resource/archive bytes remain untouched except
  the declared empty-inherits field control.
- Complete-list raw CLI controls compare the observed shared exit 239 and
  report code -17 in both modes. Retain raw channels, reports and outputs;
  do not compare whole reports, timing fields or G-code, and do not require
  an unrelated slicing outcome.
- Retain original-resource cases, the existing failure/early-exit controls,
  strict-fragment behavior, formatting, strict workspace Clippy, and fresh
  host/browser-target core and WASM checks.
