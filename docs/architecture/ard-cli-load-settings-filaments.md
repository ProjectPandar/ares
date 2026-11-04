# ARD: CLI `--load-settings` / `--load-filaments` (native argv parity)

Status: proposed. Spec + ARD only; no production change until the
E2E RED lands.

## Decision

Ares' `slice` subcommand accepts OrcaSlicer-native `--load-settings` and
`--load-filaments` argv with upstream-identical semantics, and the loaded
JSON preset files are composed into the full slicing configuration inside
`ares-core::profiles` behind a byte-oriented API (no filesystem, no
terminal): the same composition stays callable from WASM.

This is a source-cited port slice of the upstream CLI load path, not an
Ares-designed pipeline: every accepted input, ordering rule, resize/
`set_at` behavior and error exit below names its `OrcaSlicer.cpp` /
`PrintConfig.cpp` / `Config.cpp` origin. The existing `profile_entry`
byte API (catalog/selection JSON, `ProfileSliceRequest`) remains the
WASM surface; `--load-settings`/`--load-filaments` compose into the same
`ProjectSettings` input that the profile entry produces, after which the
existing slicing path runs unchanged.

## Non-negotiable constraints

- Same-argv E2E: runtime Native and Ares are invoked with identical argv
  shape on identical preset files and model; comparison is whole-file,
  generator-banner-normalized only. No comparator relaxation, no
  Ares-only normalization of the composed config.
- Error parity is part of the contract: exit codes (`Utils.hpp:22-45`),
  `record_exit_reson` payloads and stderr text match Native for every
  rejected input this slice accepts responsibility for.
- Composition order follows upstream: per-file
  `load_config_file` validation → machine → process → filaments →
  filament-colour/flush recompute (`OrcaSlicer.cpp:1953-3495`). Ares
  must not reorder merges even where an order-independent result would
  be simpler.
- Filament vector semantics are the upstream resize/`set_at` contract
  (`OrcaSlicer.cpp:3115-3337`), reusing the already-ported
  `update_values_to_printer_extruders*` miss policies
  (`PrintConfig.cpp:9677-9681`, `:9849-9863`) and the flush-matrix
  recompute (`flush_matrix.rs`) rather than new Ares arithmetic.
- No legacy fallback and no capability filter: unsupported upstream
  branches are deferred explicitly (see spec), never silently accepted.

## Ownership boundary

- `ares-cli`: argv parsing (clap flags, `;` list splitting), reading the
  referenced JSON files from disk, mapping CLI failure codes to process
  exit codes and stderr. No config semantics.
- `ares-core::profiles`: file-content validation (type/from keys),
  machine/process/filament merge with resize/`set_at` semantics,
  filament-count defaulting, filament colour + flush-volume recompute
  trigger. Pure functions over bytes/JSON; WASM-safe.
