# Direct CLI print-config overrides

## Rewrite boundary

Source: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Port the command-line input side of `libslic3r/Config.cpp:265–282,1784–1916`
(`ConfigOptionDef::cli_args`, `DynamicConfig::read_cli`) for four FFF keys,
and use the existing port of `ConfigBase::apply_only` (`:484–524`) to apply
those values. The adapter call sites are `OrcaSlicer.cpp:8203–8247`
(`CLI::setup`) and `:1352–1353,4157–4226,6924–6932` (`CLI::run`). This
adds a way to populate `ConfigOverrides`, not another configuration store
or slicing flow.

## Native option admission and parsing

- `DynamicPrintAndCLIConfig::PrintAndCLIConfigDef`
  (`libslic3r/PrintConfig.hpp:2528–2540`) combines print, action, transform
  and miscellaneous definitions. An empty `ConfigOptionDef::cli` derives
  a spelling by replacing `_` with `-`; an explicit string supplies
  `|`-separated spellings; `ConfigOptionDef::nocli` supplies none. Being a
  known project key is not enough. For example, `different_settings_to_system`
  and `extruder_variant_list` are `nocli` (`PrintConfig.cpp:2255–2258,6326–6331`).
- `cli_params` is a help metavariable (`Config.cpp:430–434`), not a hidden
  set. The print-option help section is commented out in
  `OrcaSlicer.cpp:8319–8327`; accepted print keys need not appear in help.
  The sets at `:1409,1432,1455` are separate, action-local compatibility
  allowlists, not part of `read_cli`. The inspect actions check other
  actions; `--export-settings -` checks actions and transforms and rejects,
  for example, `--slice` or `--scale`. A print override is not an action or
  transform. `CLIActionsConfigDef` (`PrintConfig.cpp:12711–12894`) owns
  `slice`, `help`, `inspect-mesh`, `inspect-paint` and `export-settings`;
  `CLITransformConfigDef` (`:12896–13062`) owns such flags as `arrange`,
  `rotate` and `scale`; `CLIMiscConfigDef` owns `load-settings`,
  `load-filaments`, and `outputdir` (`:13064–13153`). None belong in the
  print override map. `CLI::setup` classifies the action/transform options
  using their definition membership, in first-occurrence order.
- `read_cli` accepts `--key value` and `--key=value`. Native also strips a
  single leading dash; `--` ends option parsing. Repeated scalars replace
  the previous scalar. `coPercent` reads a number with optional `%` and
  writes it with `%` (`Config.hpp:1253–1272`); enum values are the defined
  serialized names, not display labels. Scalar strings do not get a second
  unescape pass after the shell. Other scalars use `set_deserialize_nothrow`
  with forward-compatibility substitution disabled.
- For `coBool` and `coBools`, bare means true; a nonempty `=value` uses
  `normalize_cli_bool_value` (`Config.cpp:1754–1782`). Native recognizes
  `1/0`, `true/false`, `yes/no`, `on/off` and `enabled/disabled`, ignoring
  case and trimming each item. A separate token after a bool flag is not
  consumed as its value. `--no-<key>` is not implicit negation: that branch
  is commented out, so a `no-` spelling must be a literal definition.
- The first vector occurrence clears its default. Repeated occurrences
  append in argv order via `deserialize(value, true)`, not by appending to
  the subsequently loaded preset. `coInts` and `coFloats` use comma-separated
  elements; `coFloats` trims elements and permits `nil` only for nullable
  definitions (`Config.hpp:932–956`). `coStrings` instead uses `;` with
  C-style unescaping inside quoted fields (`Config.hpp:1225–1229`,
  `Config.cpp:173–232`); commas in strings are not delimiters. Boolean
  vectors append true for a bare occurrence or deserialize a normalized
  comma list. Returned false or a vector deserialize exception produces
  `Invalid value for option --<key>` on stderr.
- An unknown spelling, including a print definition with no CLI spelling,
  produces `Invalid option --<spelling>`. A value-taking option with no
  value token at the end produces `Need values for option --<spelling>`.
  For `read_cli` failures, `CLI::setup` adds a blank stderr line and prints
  help to stdout; `CLI::run` adds `setup params error` on stderr and returns
  `CLI_INVALID_PARAMS` (`-2`, POSIX status `254`;
  `OrcaSlicer.cpp:1329–1333,8203–8211,8297–8332`,
  `libslic3r/Utils.hpp:26`). This path does not write `result.json` or slice
  output. Typed values rejected by the subsequent `validate(true)` instead
  use the `Params in command line error:` diagnostic at `:8228–8247`, still
  before model or preset loading. An action-local rejection after setup can
  have the same status but writes `result.json` on Linux.

## Application and output

`m_extra_config.apply(m_config, true)` and `normalize_fdm()` at
`OrcaSlicer.cpp:1352–1353` prepare the print-side input; they do not modify a
loaded project. The effective order is defaults and the saved project,
then explicitly loaded machine/process/filament presets, then the direct
CLI edits. STL has no saved project settings and uses the selected explicit
presets. The late write is `m_print_config.apply(m_extra_config, true)`
(`:4168`, after preset-loading and composition); plate config is applied
and then CLI values reapplied to the plate copy at `:6924–6926`. This is
not an object/region override. It is independent of where load flags appear
relative to print flags in argv.

The late write records `different_settings_to_system` before the next FDM
normalization (`:4157–4226`). Only supplied print keys participate. An
absent loaded key counts as changed; an equal one does not. Vectors compare
serialized elements, falling back to the first element for an out-of-width
index. Changed keys are appended once, in sorted key order, to the preset
owner's columns (process, all filament slots, or printer), preserving
existing entries. Saved-project export carries this config and dirty state
into the archive; reopening can then retain changes rather than restoring
system preset values. Archive writing itself is outside this slice.

The G-code config block serializes the applied print configuration, including
these four values and the dirty columns, through
`libslic3r/GCode.cpp:4913–4928,8272–8376` (`GCode::append_full_config`). This
is not a change to footer formatting. Linux `result.json` reports numeric
`layer_height`, `wall_loops` and `sparse_infill_density` from the global
`m_print_config`, not from the argv strings or the plate copy
(`OrcaSlicer.cpp:535–574,6924–6932,8132–8136`). The result does not dump all
print keys; `brim_type` and `nozzle_temperature` are not new result fields.

## First slice and Rust destination

| CLI spelling | Core key / source kind | Native definition |
| --- | --- | --- |
| `--layer-height` | `layer_height` / float | `PrintConfig.cpp:1134` |
| `--sparse-infill-density` | `sparse_infill_density` / percent | `PrintConfig.cpp:3691` |
| `--brim-type` | `brim_type` / enum | `PrintConfig.cpp:2116–2139` |
| `--nozzle-temperature` | `nozzle_temperature` / integer vector | `PrintConfig.cpp:8064–8071` |

Use the native-style direct `--slice <n> --outputdir <dir>` entry on a saved
FFF 3MF and on a single STL with explicit settings and filament presets.
Support both value spellings, last scalar wins, and ordered concatenation
of the integer vector's comma-separated and repeated occurrences. The
first slice exposes only the four spellings in the table; it does not
publish every registry entry as a flag.

- `crates/ares-cli/src/args.rs` owns the argv spellings and accumulation.
  Append the four value fields (or one private flattened group of those
  fields) at the tail of `DirectSliceArgs` once the direct entry is
  available. Do not add them to the `action` or `export_action` group or
  retag/replace `selection`, `inputs`, `outputdir`, or `SliceArgs::output()`.
  They are slice inputs, not action selectors; do not accept and silently
  drop them in an export-only invocation. Recheck the exact direct-entry
  symbols on the integration base before adding tests. The parsing and
  overlay design does not depend on the action refactor; literal same-argv
  execution uses the direct routes supplied by it, so do not backport a
  second dispatcher onto the subcommand-only base.
- Adapt the collected values to the existing `{key: string|string[]}`
  interface of `crates/ares-core/src/options/overrides.rs::ConfigOverrides::from_json`.
  Pass vector element tokens as array entries, with no shell or preset-path
  splitting. The registry in `options/registry` owns kinds and metadata;
  `ConfigOverrides` owns typed-wire admission and `validate_cli_config`.
  Run that preflight before file I/O. No new single-key setter is needed:
  `ConfigOverrides::apply` is internal and `SliceOptions::values()` is a
  read-only view, not an alternate override path.
- Replace the empty override argument at the direct project's
  `CliProjectSliceRequest` and at the direct STL's
  `LoadSettingsSliceRequest`; pass the same validated value set into the
  existing core requests. The saved-project loader (`project/load.rs`) and
  preset-backed composition (`project/domain/profile_input/stl_settings.rs`)
  already call `ConfigOverrides::apply`, which also uses the shared dirty
  recorder (see [dirty columns](cli-override-dirty-columns.md)). Keep all
  comparison, ownership, normalization and output logic in core. The
  existing JSON `slice --overrides` interface stays unchanged. The empty
  direct-input placeholder is the only configuration handoff replaced.
- Match the setup-failure stderr/status and lack of output for unknown and
  `nocli` direct print-option spellings and for the malformed and semantic
  controls below. Put argv diagnostics in the CLI adapter; do not turn an
  unknown spelling into a core project-load error. Do not change unrelated
  positional-argument or subcommand diagnostics. `split_native_list` in
  `main.rs` splits paths for `--load-settings`/`--load-filaments` on `;`;
  it is not the numeric vector decoder or a general `coStrings` parser.

Deferred: other FFF keys (including `wall_loops`, `brim_width` and all
bools), SLA, arbitrary aliases and single-dash spellings, unusual empty or
partial numeric tokens, nullable/string/float/point/bool vectors, automatic
registry-to-argv generation, export actions, inspect actions, multi-model
and all-plate routing, per-object edits, combined saved-3MF plus replacement
presets, and accepting direct print flags on the separate `slice` subcommand
or mixing them with its JSON overrides. Do not implement undefined upstream
parsing behavior.

## Public CLI witnesses

Run Native and Ares with **identical argument vectors** (excluding the
executable name), from separate directories containing identical input
bytes and relative filenames. Compare complete `plate_1.gcode` bytes with
only the generator banner normalized, including the config block and dirty
columns. Run each pair with the live official and seeded producers; an
unavailable or skipped Native execution cannot establish parity. Retain
argv, inputs or their hashes, statuses, stdout/stderr, result JSON when
present, and G-code outside the repository. No captured Native G-code in
fixtures and no edits to presets to simulate argv overrides.

- **Saved 3MF overrides the loaded project:** use `tests/parity/orca_cli_ender3.3mf`
  with `--slice 1 --outputdir out --layer-height 0.3
  --sparse-infill-density=30% --brim-type outer_only
  --nozzle-temperature=210,220`. Native emits `0.3`, `30%`, `outer_only`
  and `210,220` respectively, and the dirty line is
  `; different_settings_to_system = "brim_type;layer_height;sparse_infill_density";nozzle_temperature;`.
  Repeat with `--layer-height=0.24 --layer-height 0.3` and
  `--nozzle-temperature 210 --nozzle-temperature=220` in place of the two
  corresponding flags. Require the same final values and complete output.
  On Linux, compare the stable result fields: `layer_height` is the widened
  `0.3f32` (`0.30000001192092896`), `sparse_infill_density` is `30`, and
  `wall_loops` is unchanged from the no-override run.
- **Explicit presets stay below direct edits:** reuse the pre-positioned
  `cube10.stl` and the generated Ender-3 machine/process/filament preset
  files of `orca_parity/load_settings_cli.rs`. Use `--slice 1 --outputdir out
  --arrange 0 --load-settings 'machine.json;process.json'
  --load-filaments filament-0.json` with the four overrides above; place
  some direct edits before the load flags. The empty-override control uses
  the same presets and model; it must not acquire these dirty keys. Check
  the same effective values and dirty line in the G-code and the stable
  Linux result fields; `--slice` (an action excluded with `--export-settings -`),
  `--arrange`, and preset paths must remain dedicated arguments, not dirty
  keys. The Native-only classification control `--export-settings=-
  --slice 1 --outputdir out <project>` produces
  `--export-settings - cannot be combined with --slice`, status `254`,
  and Linux `out/result.json`; do not add export-combination behavior in
  this slice.
- **An equal read does not dirty the project:** on the saved fixture pass
  `--layer-height=0.2 --nozzle-temperature=220,220`. Native serializes both
  vector entries but keeps the empty dirty columns (`;;`). Include the
  full config block and unchanged report fields in the comparison.
- **Unknown, hidden and invalid input:** use the saved-project argv with
  `--not-a-print-config-key 1`, `--different-settings-to-system=x`, and
  `--layer-height=bad` in separate runs. Assert status `254`, the native
  parser diagnostic plus blank line and `setup params error`, no G-code and
  no result file. A `--layer-height=0` control instead requires
  `Params in command line error:`, `layer_height: invalid value 0.000000`,
  and `setup params error`, with the same status and no outputs. Do not
  make either failure depend on opening the model.

Start with the executable-level saved-project witness before adding the
adapter, so failure comes from refusing a valid option rather than a test
that does not build. Keep the existing core override and preset-loading
witnesses unchanged and rerun the affected cases after wiring the CLI.
