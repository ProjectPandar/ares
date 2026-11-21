# Direct CLI print-config overrides

## Rewrite boundary

Source: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Port the command-line input side of `libslic3r/Config.cpp:265–282,1784–1916`
(`ConfigOptionDef::cli_args`, `DynamicConfig::read_cli`) as a registry lookup
and dispatch by option kind. Use the existing port of `ConfigBase::apply_only`
(`:484–524`) to apply the parsed values. The adapter call sites are `OrcaSlicer.cpp:8203–8247`
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
the overridden values and the dirty columns, through
`libslic3r/GCode.cpp:4913–4928,8272–8376` (`GCode::append_full_config`). This
is not a change to footer formatting. Linux `result.json` reports numeric
`layer_height`, `wall_loops` and `sparse_infill_density` from the global
`m_print_config`, not from the argv strings or the plate copy
(`OrcaSlicer.cpp:535–574,6924–6932,8132–8136`). The result does not dump all
print keys; `brim_type` and `nozzle_temperature` are not new result fields.

## First slice and Rust destination

Use the direct `--slice <n> --outputdir <dir>` entry on a saved FFF 3MF or a
single STL with explicit settings and filament presets. Support every common
or FFF definition with a CLI spelling and a non-nullable `Float`, `Percent`,
`Int`, `Enum` or `Ints` kind. The examples below are witnesses, not an option
allowlist.

- Add one argv intake step beside `crates/ares-cli/src/args.rs`. Mirror
  `DynamicConfig::read_cli` (`Config.cpp:1784–1848`): for each long option
  not owned by clap, split the spelling from any `=value` and resolve it
  through the core registry. Use `option_definitions` and `option_metadata`
  in `options/registry.rs` for the key, kind and metadata. Generate the raw
  `ConfigOptionDef::cli` metadata from `PrintConfig.cpp` via the existing
  `scripts/gen-configdef.mjs` generator. Use `cli_args` rules to derive the
  hyphenated default name, use explicit `|`-separated spellings verbatim, and
  omit `nocli`. The preset `aliases` field and keys with underscores are
  not implicit CLI aliases. Do not maintain a second list of print keys.
- Existing clap arguments retain ownership of their spellings, including
  collisions, as the action, transform and miscellaneous definitions do
  upstream. Leave their tokens and values intact, including `--slice`,
  `--outputdir`, `--arrange`, and the preset load flags. Preserve positionals
  and the `--` terminator. Do not add a clap field per print key or change
  `selection`, `inputs`, `outputdir`, `SliceArgs::output()`, or the `action`
  and `export_action` groups. Pass one collected override set to the direct
  slice dispatch. Do not accept and discard it on an export-only route.
  Start implementation on main after the direct project and STL routes and
  their action/selection parsing have landed. Do not backport a dispatcher.
- Dispatch on kind after resolving the spelling (`Config.cpp:1843–1914`).
  For supported kinds, read `=value` or the following token. Repeated
  scalars replace earlier values. Split `Ints` on commas and append elements
  from repeated occurrences in argv order. Collect one `{key: string|string[]}`
  object with vector elements as array entries. `split_native_list` splits
  preset paths on `;`. It is not the numeric vector decoder or a general
  `coStrings` parser.
- Any other kind fails setup before consuming an operand or loading files.
  This includes bools, strings, points, other numeric and enum kinds, and
  nullable definitions. Native's `coBool`/`coBools` branches
  (`Config.cpp:1843–1849,1876–1906`) do not consume a following token. This
  slice rejects those recognized options by kind instead of treating them
  as unknown or consuming an input filename as a value. Report
  `Unsupported option --<spelling> (kind: <kind>)` followed by
  `setup params error`, with status `254` on POSIX and no output files.
  This is an explicit Ares scope diagnostic, not Native's unknown diagnostic.
- Pass the map to `ConfigOverrides::from_json` in
  `crates/ares-core/src/options/overrides.rs` for typed-wire admission, then
  call `validate_cli_config` before file I/O. No new single-key setter is
  needed. `ConfigOverrides::apply` is internal and `SliceOptions::values()`
  is read-only.
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
- Preserve Native's setup stderr, status and lack of output for unknown or
  `nocli` spellings and for the malformed and semantic controls below.
  Report argv errors at the adapter rather than as project-load errors.
  Leave unrelated positional-argument and subcommand diagnostics unchanged.

Deferred: decoding the unsupported kinds, SLA, single-dash spellings,
empty or partial numeric tokens, export and inspect actions, multi-model
and all-plate routing, per-object edits, combined saved-3MF plus replacement
presets, and direct print flags on the `slice` subcommand. Do not implement
undefined upstream parsing behavior.

## Public CLI witnesses

Run Native and Ares with **identical argument vectors** (excluding the
executable name), from separate directories containing identical input
bytes and relative filenames. Compare complete `plate_1.gcode` bytes with
only the generator banner normalized, including the config block and dirty
columns. Run each pair with the live official and seeded producers unless
specified otherwise. A skipped Native execution cannot establish parity. Retain
argv, inputs or their hashes, statuses, stdout/stderr, result JSON when
present, and G-code outside the repository. No captured Native G-code in
fixtures and no edits to presets to simulate argv overrides.

The four primary values are defined in `libslic3r/PrintConfig.cpp`:
`layer_height` (`:1134`), `sparse_infill_density` (`:3691`),
`brim_type` (`:2116–2139`), and `nozzle_temperature` (`:8064–8071`).

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
- **Another key of a supported kind uses the same intake:** on the saved
  fixture pass `--wall-loops=3` (`coInt`, `PrintConfig.cpp:5968`). Run the
  same-argv whole-G-code comparison in official mode. Require
  `; wall_loops = 3`, `; different_settings_to_system = wall_loops;;`, and
  Linux `result.json` `wall_loops: 3` instead of the baseline `2`. No
  `wall_loops` argument field or key-specific parser branch is needed.
- **A known spelling of a deferred kind fails setup:** on the same direct
  entry pass `--enable-support=0` (`coBool`, `PrintConfig.cpp:7316`). This
  Ares-only control requires the `bool` kind diagnostic, setup failure,
  status `254`, and neither G-code nor `result.json`. Native supports the
  kind, so this is not a same-outcome comparison.
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
