# First-layer bed-temperature export compatibility

This source-cited rewrite covers the applied compatibility value emitted in the
full configuration footer. It is separate from the already ported machine-start
M140 reader and does not redesign temperature selection.

## Pinned upstream ownership

The pinned upstream is `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `src/libslic3r/PrintConfig.cpp:13553–13560` declares the bed vectors,
  `bed_temperature_initial_layer_single`, and the legacy-compatible
  `first_layer_bed_temperature` vector. Bed-type selection is represented by
  `get_bed_temp_1st_layer_key` in `PrintConfig.hpp:755–778`.
- `src/libslic3r/Print.cpp:3773–3784` obtains the non-sequential
  `ToolOrdering`, selects its first extruder, and records first-layer tools.
  `Print.cpp:3805–3813` publishes the first-layer used set for the ordinary
  non-sequential path. Sequential collection at `Print.cpp:3676–3691` is a
  separate producer and remains outside this slice.
- `src/libslic3r/GCode.cpp:3568–3687` derives `initial_extruder_id` from the
  ordering and computes the compatibility scalar. With
  `BedTempFormula::btfFirstFilament`, `GCode.cpp:3682–3687` reads the selected
  initial filament. With `btfHighestTemp`, it calls
  `get_highest_bed_temperature(true, print)`.
- `src/libslic3r/GCode.cpp:5497–5513` defines the two readers. The ordinary
  reader indexes the bed-type vector by filament; the highest-temperature
  reader considers `print.get_slice_used_filaments(true)`, not the complete
  configured palette.
- `src/libslic3r/GCode.cpp:4884–4892` writes the non-BBL compatibility footer
  line `first_layer_bed_temperature`. This line uses the selected compatibility
  scalar, while the machine-start placeholders at `GCode.cpp:3991–4004` retain
  their complete vectors and scalar selected by the same reader.
- `src/OrcaSlicer.cpp:4218–4259` is the loaded-config synchronization boundary:
  FFF defaults are applied into the loaded print config, the loaded config is
  applied back into the full config, and filament values are normalized to
  variants. Temperature selection occurs later in GCode export; this boundary
  must remain ordering-compatible and must not synthesize a late footer value.

## Rust destination and replacement boundary

The existing applied full-config writer is
`crates/ares-core/src/options/config_export/writer.rs::write_runtime_tail`.
It currently emits `first_layer_bed_temperature` from the first element of the
bed vector. Replace that settings-only projection by threading the result of
`project_slice/gcode_emit/machine/temperature.rs::first_layer_bed_temperature`
through `InitialFilaments::initial_bed_temperature` to `write_runtime_tail`.
There must be no second implementation of the formula. The shared reader
already preserves indexed front-on-miss and first-layer-used semantics.
`GCode.cpp:3681–3685` takes this compatibility snapshot before the object loop;
the Rust export config must be materialized with the same initial snapshot,
not an object-local or final writer-cache temperature. Early template config
has no ordering context and omits this scalar until export materialization.

The existing ordering and synchronization machinery supplies the context. No
new selector, palette broadcast, physical-nozzle index, or rendered-token edit
is allowed. The generic full-config writer, vector order, nil handling, and
sorted key emission remain unchanged.

## Included behavior

- Select the bed-type initial-layer vector using the same bed-type key mapping
  as `get_bed_temp_1st_layer_key`.
- For the first-filament formula, use the actual initial printing filament
  selected by the ordinary ToolOrdering path, not vector position zero.
- For the highest-temperature formula, reduce only the first-layer used set,
  matching `get_highest_bed_temperature(true, print)`.
- Emit the selected scalar in the existing full-config compatibility line while
  preserving all complete vector entries elsewhere.
- Preserve the loaded-config synchronization ordering and the distinction
  between the compatibility footer and machine-start M140/M190 emission.

## Deferred behavior

Sequential first-layer collection, mixed-filament expansion, dynamic selector
ordering, multi-nozzle grouping, BBL-specific footer layout, GUI preset
lifecycles, plate read-back, and ordinary-layer temperature transitions remain
outside this rewrite. The machine-start reader, custom M140/M190 suppression,
and complete output parity are not reopened by this slice. The existing
filament volume-map and mixed-metadata slices remain separate.

## Qualification contract

The two unchanged authored inputs are retained:

- `tests/parity/m140-initial-bed/nonzero-initial-tool.3mf`, SHA256
  `44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a`.
- `tests/parity/m140-initial-bed/unused-hotter-filament.3mf`, SHA256
  `893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0`.

The nonzero-initial-tool witness uses the actual initial filament and expects
`first_layer_bed_temperature = 73` in Native, API, and CLI output; the previous
Ares footer emitted `55`. The unused-hotter-filament witness excludes unused
filaments from the highest-temperature reduction and expects
`first_layer_bed_temperature = 63` in all three outputs; the previous Ares
footer emitted `88`. Both witnesses require stable Native first/replay output,
successful API and CLI calls, and raw API/CLI equality. The labelled scalar
line is compared exactly. Complete output bytes and their exact first
difference are retained separately; no whole-output parity is claimed.
