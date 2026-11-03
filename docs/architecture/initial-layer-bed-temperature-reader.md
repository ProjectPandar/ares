# ARD: initial-layer bed-temperature reader for machine start

This document defines the source-backed initial-layer bed-temperature reader
boundary. The export compatibility snapshot and complete-output parity are
separate concerns.

## Owning upstream boundary

Pinned OrcaSlicer: `345efec64c7b95a05ce5cf96df0dde66a158835c`; source paths below
are relative to that revision.

- `src/libslic3r/GCode.cpp:3990–4005`, `_do_export`: the bed type chooses the
  initial-layer temperature vector at3992. `target_bed_temp` is initialized
  at3994; HighestTemp calls `get_highest_bed_temperature(true, print)` at3996;
  FirstFilament calls `get_bed_temperature(initial_extruder_id, true,
  curr_bed_type)` at3998. The scalar placeholder
  `bed_temperature_initial_layer_single` receives that value at4003. The
  `bed_temperature_initial_layer` and `bed_temperature` vector placeholders at
  4001–4002 remain complete vectors, not this scalar projection.
- `GCode.cpp:5497–5502`, `get_bed_temperature`: use the bed-specific option and
  `ConfigOptionInts::get_at(extruder_id)`, preserving source front-on-miss.
- `GCode.cpp:5504–5513`, `get_highest_bed_temperature`: maximum initialized to
  zero over `print.get_slice_used_filaments(true)`, not the whole configured
  palette. `src/libslic3r/Print.hpp:1241–1247` stores the first-layer-used set
  separately from whole-print usage.
- `GCode.cpp:5518–5540`, `_print_first_layer_bed_temperature`: HighestTemp at
  5526–5527 uses that same first-layer set; FirstFilament at5530 reads the
  supplied `first_printing_extruder_id`, not column0. The custom-G-code scan
  at5534 detects M140/M190. The disabled `#if0` block does not permit
  replacing the selected source temperature with a parsed custom value.
- `GCode.cpp:5541–5545`: always update writer bed-temperature state with the
  selected value and `wait`; emit the generated command only when custom
  G-code did not already set temperature. Calls at4359 and4672 supply the
  relevant first printing filament for export/object startup.

This is a source-cited libslic3r reader/context rewrite, not an Ares temperature
pipeline. Export-footer snapshot behavior is deferred.

## Existing shell and exact Rust destination

The existing `project_slice/gcode_emit/placeholders.rs::
insert_bed_temperature_placeholders` uses `value.index(0)` for FirstFilament and
reduces the whole list for HighestTemp. The machine-start template consumes
that scalar. `gcode_emit/machine/temperature.rs::first_layer_bed_temperature`
likewise uses first/all values without the source reader context;
`append_startup` uses that result around custom M140/M190 suppression.

Destination: those two readers and the minimum argument/context plumbing in
`gcode_emit/machine.rs` and its existing start/template callers. Reuse the
prepared schedule/initial-filament state and source-equivalent first-layer
usage; do not change ToolOrdering algorithms or infer usage from an exported
footer. The current settings-only scalar shell is replaced, not retained as a
fallback. Do not change indexed vector placeholders or generic parser syntax.

## Included and non-negotiable behavior

- Bed-type option selection; FirstFilament's supplied initial/first-printing
  logical filament; HighestTemp restricted to actual first-layer usage.
- The scalar machine-start placeholder and initial bed-command reader, with
  their upstream contexts retained separately. Do not assume a print-wide
  snapshot is always the same as an object-local startup filament.
- Preserve source get_at semantics, full input vectors/order/lengths/nils,
  existing mapping and exact external comparisons. Do not substitute physical
  nozzle indices or a hardcoded expected tool.
- Preserve wait/non-wait choice, custom-command suppression and writer state.
  No duplicate M140/M190 or string replacement in emitted G-code.
- Core stays filesystem/environment/UI-free and portable to browser WASM,
  Windows, macOS and Linux. No new platform dependency or legacy fallback.

## Deferred and excluded

The carried `first_layer_tools = ordering.layers.first().extruders` is literal
only to the **non-sequential** producer in `Print.cpp:3800–3808`, which uses
`layer_tools().front().extruders`. It is not a rewrite of the sequential
producer. Deferred: `Print.cpp:3676–3691` sequential-print collection, which
unions the first layer of each object's ToolOrdering and sorts/deduplicates the
result. Also deferred: `expand_mixed_filaments`, the mixed-filament expansion
of that used-filament set, tied to the separate `filament_is_mixed` slice.
The two witnesses exercise the non-sequential, non-mixed producer only.

`filament_is_mixed` is a separate small coBools resize/echo slice:
`PrintConfig.cpp:3545` declaration and11481 resize. Its observed `0,0` versus
`0` first difference is preserved, explicitly deferred, and not implemented
here. Do not fix vector echo by forcing defaults or changing the comparator.

Also deferred: export compatibility-temperature snapshots, nozzle temperature
readers, ordinary-layer bed changes, variant/group solvers, first-layer usage
producer rewrites, BBL identity detection, new thermal command scanning,
preset refresh, dirty-list bookkeeping, registries and all inner_wall_speed
work. Existing custom-command scanning behavior is unchanged. Scoped reader parity does not establish complete-output parity.

## Failure inventory and public acceptance contract

The two authored inputs change only project settings of
`chamber-used-extruder/chamber-t1-equal.3mf`; neither is a Native export. The
input bytes and complete vector values are preserved.

| Input | SHA256 | Recorded first difference |
|---|---|---|
| nonzero-initial-tool | `44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a` | byte733,line26: Native M140 S73; API/CLI M140 S55 |
| unused-hotter-filament | `893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0` | byte733,line26: Native M140 S63; API/CLI M140 S88 |

Case2 discriminates initial tool1 from slot0 (bed values55/73). Case4
HighestTemp discriminates first-layer usage from the whole palette (88/63,
with hotter unused slot0). Native premises require actual
positive model activity and complete input vectors, not just comments. Reject
any test whose selected tool/usage/formula premises fail.

Public E2E acceptance requires Native first plus exact replay under the
existing generator-only normalization, successful public API, actual CLI
exit0 and raw API=CLI. The labelled oracle compares executable start through
the first M140/M190 and rendered uses of
`bed_temperature_initial_layer_single`; it does not claim whole-output parity.
Unchanged strict whole-output comparisons retain exact first differences and
all raw bytes. Failed setup or Native premises do not establish reader parity. No
Native-export-derived Ares inputs, tolerance or scalar/vector equivalence is
permitted.

Implementation changes only these readers and minimum context plumbing.
Identical input bytes must retain the next strict first difference without
weakening it: filament_is_mixed or compatibility snapshot residuals may prevent
whole-output parity. Scoped reader parity must be labelled as such, never
whole-output parity.
