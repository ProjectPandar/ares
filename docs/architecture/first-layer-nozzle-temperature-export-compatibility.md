# First-layer nozzle-temperature export compatibility

## Source boundary

Pinned OrcaSlicer revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This is a libslic3r GCode export snapshot rewrite, not a new temperature
selection pipeline.

- `src/libslic3r/PrintConfig.cpp:4116–4123` registers
  `nozzle_temperature_initial_layer` as coInts, default 200. Preserve the full
  filament-indexed vector; the compatibility scalar is not its replacement.
- `src/libslic3r/GCode.cpp:3568–3678` establishes the export-initial logical
  filament from ToolOrdering. The existing initial-filament producer is reused,
  not replaced with a new selection policy.
- `GCode.cpp:3681–3687` explicitly captures both compatibility temperatures
  before the object loop can reassign `initial_extruder_id`. The nozzle value
  is `print.config().nozzle_temperature_initial_layer.get_at(
  get_filament_config_index((int) initial_extruder_id))`. It is neither the
  maximum palette temperature nor the final active-tool temperature.
- `GCode.cpp:5039–5053` owns filament-to-config indexing. With a Print, the
  current-layer cache delegates to `Print::get_filament_config_indx`; without
  one, the logical filament ID is the config column.
- `src/libslic3r/Print.cpp:5068–5071,5114–5147` owns that config index and its
  grouping/variant lookup. The no-grouping branch returns the logical filament;
  grouped lookup uses volume/extruder type and filament variant/self-index
  metadata. Do not replace all branches with physical-nozzle indexing or assume
  grouped columns always equal logical filament IDs.
- `src/libslic3r/Config.hpp:676–680` defines get_at: index the requested column
  when present, otherwise use the vector front. Nonempty vectors are an
  upstream invariant.
- `GCode.cpp:4884–4895` writes the ordinary non-BBL full-config tail. At 4894,
  `first_layer_temperature` receives the captured nozzle scalar, after the bed
  value and bed shape, before first-layer height.
- `GCode.cpp:4294` uses the same snapshot in the BBL configuration block. A BBL
  layout or identity rewrite is not part of this slice.

## Rust destination and existing shell

`crates/ares-core/src/options/config_export/writer.rs::write_runtime_tail`
currently takes the first entry of the runtime nozzle-temperature vector for
`first_layer_temperature`. Replace that settings-only projection with an
explicit export-initial snapshot argument, alongside the bed snapshot already
threaded through `write_config_block` by `gcode_emit/orchestration.rs`.

Reuse `gcode_emit/machine.rs::initial_layer_nozzle_temperature` and
`machine/temperature.rs::filament_int`, rather than implementing another
indexed-temperature reader. They already preserve front-on-miss. The existing
`options/project_variants/index.rs` and `select.rs` materialize the resolved
variant boundary. Before implementation, verify how their full/runtime views
represent the source config column for the unchanged static witnesses. Pass
that resolved column to the shared reader; do not invent a selector, silently
assume an identity map in grouped cases, or broaden variant mapping behavior.

The scalar must be captured with the export-initial filament before object
emission and retained independently of object-local transitions and writer
cache updates. Existing machine-start commands, complete vector placeholders,
config-key ordering and bed snapshot behavior remain unchanged. No emitted
text replacement or alternate default-temperature path is permitted.

## Included and deferred behavior

Included: the ordinary static non-sequential initial-filament witnesses,
existing resolved config-column semantics, shared indexed nozzle reader,
pre-object-loop snapshot and the compatibility scalar writeback. Preserve
input vectors, ordering, types and source get_at behavior. Core remains free
of filesystem, environment and UI dependencies on all Tier1 platforms.

Deferred: new grouping/variant solvers, dynamic/layered index changes, mixed
expansion, sequential collection, nozzle command generation, standby/idle
thermal state, custom-command handling, GUI/plate read-back, registry changes
and BBL layout work. Existing scaffold is only a temporary config-export shell
around the cited GCode snapshot, not an Ares-owned temperature design.

## Witnesses and expected values

Use the unchanged authored project inputs:

- `tests/parity/m140-initial-bed/nonzero-initial-tool.3mf`, SHA256
  `44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a`.
- `tests/parity/m140-initial-bed/unused-hotter-filament.3mf`, SHA256
  `893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0`.

Both Native outputs have `first_layer_temperature = 223`; the existing Ares
outputs have `211`, at line6571,column30. Both require stable Native replay,
positive tool1 model activity, successful public API and CLI output and raw
API/CLI equality. Compare the unique labelled footer line exactly. Preserve
complete outputs and exact first differences separately; no whole-output
parity is claimed. The existing bed footer and map witnesses must retain their
values. Wrong config-column selection, slot-zero projection, front-on-miss
regression, late object-local recapture, and mutation of complete vectors are
the failure modes this boundary must distinguish.
