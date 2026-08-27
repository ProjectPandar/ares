# Spec: OrcaSlicer printer smoke and option coverage parity

## Observable contract

The parity seam extends from the single `ksr_fdmtest_v4` project to the full
upstream profile matrix. `tests/parity/` is a new test suite owning model
fixtures (binary STL cubes) shared by the parity harness. The external seam
stays `slice_project(3mf_bytes, metadata)` versus a locally executed
OrcaSlicer 2.4.2 CLI:

1. **Printer smoke** — every FDM machine preset under
   `OrcaSlicer/resources/profiles/<vendor>/machine/*.json` is flattened
   (inherits chains resolved), combined with its `default_print_profile` and
   `default_filament_profile`, exported to a 3MF by the OrcaSlicer CLI, and
   sliced by both OrcaSlicer and Ares. The two G-code streams must satisfy the
   existing semantic comparison (`ksr_fdmtest_v4/semantic::compare`):
   exact deposited motion multisets, lifecycle events, control events,
   templates, statistics, timing within bounds.
2. **Option coverage** — every option in the `tests/ksr_fdmtest_v4/
   options-v242.json` inventory is varied around the smoke-test baseline:
   - Boolean: both values.
   - Enum: every legal value.
   - Range (float/int/percent): min, max, and one seeded-random interior
     value.
   Each variation rebuilds the 3MF through the OrcaSlicer CLI with the option
   applied, slices with both slicers, and applies the same semantic
   comparison.
3. **Loader compatibility** — Ares must load every 3MF that OrcaSlicer's own
   CLI produces, including CLI exports that reference plate thumbnails in
   `_rels/.rels` without embedding the PNG parts.

The harness is environment-gated: it runs only when `ARES_ORCA_BIN` names an
executable OrcaSlicer 2.4.2 command (a wrapper that assembles the host library
path is acceptable) and the `OrcaSlicer/resources/profiles` tree exists.
Otherwise the tests skip, so Tier 1 CI without the reference binary still
passes. Reference G-code produced by the OrcaSlicer CLI is cached under
`target/parity-cache/` keyed by the SHA-256 of (flattened presets, override,
model, orca version) so repeated runs do not re-slice unchanged cases.

## Upstream boundaries

1. `OrcaSlicer/src/OrcaSlicer.cpp:1951-2135` (`load_config_file`,
   `--load-settings`, `--load-filaments`) — CLI profile loading and the
   `from`/`type` semantics the flattener must reproduce.
2. `OrcaSlicer/src/libslic3r/Preset.cpp:1360-1400` — printer preset option
   ownership used to decide which flattened preset owns an overridden option.
3. `OrcaSlicer/src/libslic3r/PrintConfig.cpp` — option defaults, min/max, and
   enum values that define the coverage sweep domain.
4. `OrcaSlicer/src/libslic3r/Print.cpp:1700-1740` — validation behavior that
   the CLI applies before slicing (e.g. relative-E `G92 E0` rule); the
   harness must feed presets that OrcaSlicer itself accepts.
5. OrcaSlicer 3MF package writing (`--export-3mf`) — the plate metadata and
   root relationships the Ares loader must tolerate.

## Incremental acceptance

1. Ares loads OrcaSlicer CLI-exported 3MFs whose thumbnail relationships lack
   parts (red test on a captured CLI export first).
2. `tests/parity/` owns cube STL fixtures; a single-printer smoke test
   (Ender-3) reaches semantic parity or reports its first divergence.
3. The vendor-wide smoke sweep runs every FDM machine preset and produces a
   pass/fail inventory committed as a tracked summary.
4. The option sweep covers every inventory option per the value rules above
   and produces a tracked summary of the first divergence per option.
5. Each divergence fixed lands as its own source-cited slice with a red test,
   keeping files below 400 LOC, tests in dedicated mods, and no
   `include!`/`include_bytes!` splitting; obsolete source-pinning tests are
   deleted as their observable behavior is covered.
