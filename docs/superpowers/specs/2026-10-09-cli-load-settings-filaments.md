# Spec: CLI `--load-settings` / `--load-filaments` port slice

Date: 2026-10-09 · Status: PROPOSED (no RED yet)
Upstream: pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/OrcaSlicer.cpp`, `src/libslic3r/PrintConfig.cpp`,
`src/libslic3r/Config.cpp`, `src/libslic3r/Utils.hpp` (line numbers below
verified against that pin) ·
ARD: `docs/architecture/ard-cli-load-settings-filaments.md`

## Goal

`ares slice` accepts `--load-settings "machine.json;process.json"` and
`--load-filaments "filament-0.json;filament-1.json"` with upstream-identical
composition and error behavior, verified by same-argv runtime-Native E2E.

## Relationship to the existing profile entry (single composition)

The upstream merge below (`OrcaSlicer.cpp:1953-3495`) is THE composition
boundary this slice ports. `ares-core::profiles` gains exactly one
merge implementation shaped like it. The existing
`entry.rs::ProfileSliceRequest` (catalog/selection JSON) keeps its public
byte API, but its internal composition is refactored to call that same
merge — its vendor catalog/selection parsing remains an input adapter
that produces preset fields, not a second composition. Any existing
overlap in the current private composition logic is replaced in this
slice, not kept in parallel; where the first GREEN cannot finish that
fold-in, the overlapping shell is explicitly marked temporary in-code
and removed before the slice closes. Two parallel compositions are not
an acceptable end state.

## Upstream boundary (source-cited)

### Option registration and value shape

- `PrintConfig.cpp:10890-10895` — `"load_settings"` (coStrings,
  `cli_params "\"setting1.json;setting2.json\""`), same block
  `:10896-10901` — `"load_filaments"` (coStrings, `;`-separated list).
- Values are read once at `OrcaSlicer.cpp:1344` (`load_settings`) and
  `:1352` (`load_filaments`) as `ConfigOptionStrings`; the `;` split is
  part of the coStrings CLI value shape.

### Per-file validation — `load_config_file` lambda (`OrcaSlicer.cpp:1953-2024`)

- `DynamicPrintConfig::load_from_json` (`Config.cpp:811-816`) with
  `ForwardCompatibilitySubstitutionRule::Enable` (`:1960`, BBS-fixed).
- `key_values`: `name` (`BBL_JSON_KEY_NAME`), `from`, `type`,
  `filament_id`.
- `from` must be `system|User|user`, else stderr
  `":file %1%'s from %2% unsupported"` → `CLI_CONFIG_FILE_ERROR` (`:1989-1993`).
- `type` must be `machine|process|filament`, else
  `": unknown config type %1% of file %2% in load-settings"` →
  `CLI_CONFIG_FILE_ERROR` (`:1996-2000`).
- JSON load failure → `CLI_CONFIG_FILE_ERROR` (`:1966-1969`, `:2021-2023`).
- `config.normalize_fdm()` per loaded file (`:2005`).

### Machine/process loop (`OrcaSlicer.cpp:2025-2106`)

- Duplicate machine file → `"duplicate machine config file: "` stderr +
  `CLI_CONFIG_FILE_ERROR` (`:2036-2040`); duplicate process likewise
  (`:2073-2077`).
- `printer_settings_id`/`print_settings_id` set to the file's `name`
  (`:2043`, `:2083`); non-`system` presets keep their `inherits` value as
  the system name (`:2040-2042`, `:2080-2082`).
- `default_print_profile` captured from machine (`:2054-2056`);
  `compatible_printers` captured from process (`:2084`).
- Printer-technology conflicts across any two loaded configs →
  `CLI_INVALID_PRINTER_TECH` (`:2101-2106`, same check for filaments at
  `:2178-2186`).

### Filament loading (`OrcaSlicer.cpp:2113-2215`)

- Per file: type must be `filament`, else
  `": unknown config type %1% of file %2% in load-filaments"` →
  `CLI_CONFIG_FILE_ERROR` (`:2137-2141`, `:2175-2179`).
- `use_first_fila_as_default` block (`:2121-2170`): first non-empty
  entry becomes the default filament config; empty list entries reuse
  that default (name/id/config/inherit; `:2202-2214`);
  count>0 but no loadable default → `CLI_CONFIG_FILE_ERROR` (`:2166-2170`).
- `load_filament_count` = list size (`:2113`); when unset,
  `filament_count` defaults to it (`:2270-2271`).

### Merge into the accumulated print config

- Input files seed `m_print_config` first: per input file
  `config += std::move(m_print_config); m_print_config = std::move(config)`
  (`:1870-1871`), so an embedded-3MF project config is overlaid by the
  accumulated print config (and, for a bare STL, the loaded presets ARE
  the config).
- Machine merge `:2826-2954`, process merge `:2956-3110` via
  `update_full_config` (`:2744+`), `compute_variant_index`
  (`:2707-2742`), `load_default_gcodes_to_config` (`:2850`, `:2994`),
  `different_settings`/`inherits_group` bookkeeping sized
  `filament_count+2` (`:2817-2818`).
- Filament merge `:3115-3337` (the resize/`set_at` contract):
  - `filament_self_index` default init 1..n (`:3119-3126`);
    `old_start_indice`/`old_variant_counts` derived from it (`:3128-3140`).
  - `filament_settings_id`: `resize(filament_count, name)` then
    `set_at(name, filament_index-1, 0)` (`:3152-3160`).
  - `filament_ids`: same resize/`set_at` shape (`:3173-3177`).
  - `filament_extruder_variant` default init (`:3179-3185`) and variant
    index mapping against `old_start_indice` windows (`:3187-3199`).
  - Per-key application: scalar keys skipped except
    `compatible_printers_condition`/`compatible_prints_condition`
    (`:3259-3280`); vector keys `set_at(src, filament_index-1, 0)`,
    variant keys via `set_with_restore_2` (`:3281-3308`); missing
    option definitions → `CLI_CONFIG_FILE_ERROR` (`:3293-3298`).
  - `filament_self_index` rebuilt from `new_variant_counts` (`:3326-3337`).
  - This reuses the already-ported variant-miss policies
    (`PrintConfig.cpp:9677-9681`, `:9849-9863`) — no new resolution rules.
- Filament colour + flush recompute `:3339-3495`:
  `filament_colour` created (`resize(filament_count, "#FFFFFF")`) only
  when the project has none and the CLI override supplies one
  (`:3341-3348`); colour overlay (`:3356-3404`);
  `filament_is_support` count mismatch → `CLI_CONFIG_FILE_ERROR`
  (`:3413-3418`); `flush_volumes_matrix`
  `resize(n² × extruder_count)` (`:3429-3432`), `flush_multiplier`
  `resize(extruder_count, 1)` (`:3435-3436`), per-nozzle
  `nozzle_flush_dataset` via `get_index_for_extruder` (`:3438-3466`),
  `FlushVolCalculator` matrix fill (`:3468-3492`) — the existing
  `flush_matrix.rs` port.

### Error codes (`Utils.hpp:22-45`)

`CLI_SUCCESS 0`, `CLI_INVALID_PARAMS -2`, `CLI_FILE_NOTFOUND -3`,
`CLI_CONFIG_FILE_ERROR -5`, `CLI_INVALID_PRINTER_TECH -7`; exits via
`record_exit_reson` + `flush_and_exit` (`OrcaSlicer.cpp:2046-2048` etc.).
The existing `stock_validation` harness already pins native exit codes
(`-18`/`-51` rem_euclid(256), `stock_validation.rs:108-124`) — the new
flags' codes are pinned the same way.

## Rust destination

- `ares-cli` (`src/main.rs` `SliceArgs`): new clap args
  `--load-settings <a;b>` and `--load-filaments <a;b;c>` (`;`-split),
  file reads, exit-code/stderr mapping through the existing
  `CliFailure` path. Argv only — no config semantics.
- `ares-core::profiles`: the single byte/JSON composition described
  above (pure, WASM-safe), producing the `ProjectSettings`-level input
  the slicing path consumes. Reuses `project_variants` (variant/miss
  policies), `flush_matrix.rs` (colour/flush recompute) and the existing
  `composition` module internals it replaces where they overlap.
- `ares-wasm`: unaffected (no new surface; the byte API is shared).

## Included

1. `--load-settings` machine+process: exactly one of each, duplicate /
   unknown-type / bad-`from` / bad-JSON errors with upstream text and
   `CLI_CONFIG_FILE_ERROR`; `printer_settings_id`/`print_settings_id`
   naming; `inherits` capture for user presets.
2. `--load-filaments`: N-slot lists, empty-slot default-filament reuse
   (`use_first_fila_as_default`), `filament_count` defaulting,
   `filament_settings_id`/`filament_ids`/`filament_extruder_variant`/
   `filament_self_index` resize/`set_at` merge, scalar skip rules.
3. Filament colour + flush-volume recompute trigger when the composed
   config requires it (existing `flush_matrix.rs` path).
4. Printer-technology conflict errors.
5. Same-argv E2E harness + controls (below), including error-path
   exit-code/stderr parity assertions.

## Deferred (explicit; none of these flags exist in Ares' CLI — clap
rejects them with an unknown-argument error, matching the "not
implemented, not silently accepted" rule)

- `--export-3mf`, `--arrange`, `--uptodate-settings`,
   `--uptodate-filaments`, `--downward-check`, upward/downward
   compatibility fetching, BBL `machine_full`/`process_full`
   `model_id` lookups (`OrcaSlicer.cpp:2050-2066`, `:2296-2360`).
- `input_obj_colours` auto-color fill (`:2217-2268`) — needs a colored
   OBJ witness first; its `filament_colour` resize/render block is cited
   for later reuse.
- `clone_objects`/`skip_objects`/`loaded_filament_ids` interactions
   (`:1567-1589`, `:1801-1803`) — separate object-listing slice.
- GUI load paths (`GUI_App.cpp`).

## E2E plan (RED first; the only RED is a compiling, failing E2E test)

Harness: `orca_parity` test(s) alongside `stock_validation`, reusing its
preset reconstruction (vendor machine/process/filament JSON →
`--load-settings`/`--load-filaments` inputs, `stock_validation.rs:19-58`):

1. **Positive single-tool**: Ender-3 (or Kobra S1 Max) machine+process +
   one filament + `cube10.stl`; native argv
   `--slice 0 --load-settings m.json;p.json --load-filaments f0.json
   --outputdir <dir> cube10.stl` vs identical `ares slice` argv;
   whole-file banner-normalized byte equality.
2. **Positive multi-filament**: two distinct filament presets (2 slots),
   exercising resize/`set_at` and flush recompute.
3. **Empty-slot default reuse**: `f0.json;;`-shaped list → default
   filament fills slot 1 (upstream `:2202-2214`).
4. **3MF input + `--load-settings` control**: an embedded-project 3MF
   (existing parity fixture) sliced with `--load-settings` overrides,
   covering the `:1870-1871` project-config overlay order feeding
   `update_full_config`.
5. **Error parity**: duplicate machine file; unknown `type`; bad `from`;
   unloadable JSON — each pins native stderr text + exit code and the
   identical Ares failure (no reference stream exists).
6. **Existing load-settings cases**: the `stock_validation` matrix cases
   whose Native exits normally (success or error code) run through the
   same-argv harness; VENDOR_INCOMPLETE preset-missing errors keep
   error-text parity. Native-crash shapes (e.g. the Cubicon SIGABRT
   family, `o8_cubicon_empty_vector.rs` vector shape) are documented
   only — no runtime test, per the established native-crash rule.
7. Modes: strict in both seeded and official for non-`random()` printers
   (the RNG SKIP rule from the stock sweep applies unchanged if a
   random()-start-gcode printer is ever added).

RED shape: one same-argv E2E RED — the ares argv does not exist today,
so the control fails at arg parsing while native produces the reference;
the RED commit compiles (test-only change). No separate core-API RED.
