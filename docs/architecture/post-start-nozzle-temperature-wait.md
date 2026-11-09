# Post-start first-layer nozzle temperature wait

## Source boundary and Rust destination

Upstream is `OrcaSlicer@345efec64c7b95a05ce5cf96df0dde66a158835c`.
This is a Rust rewrite slice of `libslic3r/GCode.cpp:4439-4441` and
`GCode::_print_first_layer_extruder_temperatures` at `:5553-5585`, not
an independently designed startup or thermal pipeline.

The call runs for `is_bbl_printers`, after filament-start/initial pressure
advance and before writer preamble, with `wait = true`. Its template
predicate is `custom_gcode_sets_temperature` at `GCode.cpp:780-840`.
The existing non-waiting first-layer nozzle emission and command formatter
are compatibility shells for this same upstream helper/writer boundary.

Rust destinations:

- `gcode_emit/machine.rs::append_start`: obtain the nozzle-temperature
  predicate from the already-rendered machine-start string; do not render
  the template twice or inspect the unexpanded source instead.
- `gcode_emit/motion/state.rs::EmitState`: retain only that boolean across
  startup phases, rather than retaining another copy of the rendered text.
- `gcode_emit/machine/temperature.rs`: reuse its source-ported scanner and
  first-layer nozzle selection; thread the Native `wait` operand instead
  of hard-coding non-waiting emission.
- `gcode_emit/layer_gcode/startup.rs::append_print_preamble`: invoke the
  waiting counterpart inside the existing BBL branch immediately before
  `machine::append_flavor_preamble`.
- Reuse `gcode_emit/temperature.rs` for `GCodeWriter::set_temperature`
  formatting; do not invent another M104/M109 formatter.

## Included behavior

1. Preserve the existing pre-start non-waiting path. Add only the missing
   post-start BBL call; it is independent of `enable_pre_heating` and does
   not apply to non-BBL startup merely because a template is empty.
2. Scan the rendered start using the existing equivalent of
   `custom_gcode_sets_temperature(..., 104, 109, include_g10, ...)`.
   Preserve leading space/tab handling, uppercase commands, signed decimal
   `strtol` command parsing, compact parameters, comment/CR/LF/NUL
   boundaries, and the original conditional precedence. Recognized M104
   or M109 suppresses fallback even without a parsed S value. G10 is
   temperature-setting only under the RepRapFirmware operand and after
   successful S parsing. Do not replace this with substring/token matching.
3. When the template controls nozzle temperature, emit no fallback.
   Otherwise preserve the existing SEMM/all-printing-filaments allocation,
   first-layer temperatures, non-first-tool idle/standby adjustment,
   positive-temperature condition, and tool-suffix formatting.
4. Carry `wait = true` through the existing formatter so the post-start
   fallback is M109, while the existing non-waiting path remains M104.
   Preserve Native order relative to filament-start, pressure advance,
   preamble, and the spaghetti-detector command.
5. Preserve the completed machine-start marker producer unchanged.

## Deferred adjacent behavior

`GCodeProcessor.cpp:1618-1968` usage blocks, pre-heating/pre-cooling
injection, scheduling, and consumer behavior remain deferred and are not
part of this helper call. Do not add a thermal scheduler, an independent
writer-temperature cache, new tool selection, new filament allocation,
new profile defaults, or a broader startup rewrite. Existing emitter/tool
collection scaffolding remains a temporary compatibility shell, not a
claim of complete GCodeWriter or thermal-runtime parity.

## Existing failure and test-first sequence

At `67c9b23213b300da3d01d94b56cc826485aa81b6`, the unchanged public test
`runner::profile_entry_3mf::machine_start_end_marker::enabled_empty_start_still_emits_the_marker_complete_output`
fails in both official and seeded modes at byte 31863, line 780, column 1:
Native emits `M109 S220 ; set nozzle temperature and wait for it to be reached`,
while Ares emits `G90`. Its Native premise has an effective true flag,
empty rendered start, and the correct marker. Both API and CLI admit the
input. The marker family remains honestly 11 PASS / 1 FAIL at that exact
revision; those receipts must not be relabeled.

After this SPEC, rerun that same existing test with fresh matched-catalog
Native binding before production; no new fixture, modified positive
input, comparator change, or assertion relaxation is needed. Then port
only the boundary above. Require all twelve existing public Native/API/CLI
complete ordered-byte cases PASS in both modes with generator-identity-only
normalization. Recheck the same 25 existing adjacent/smoke IDs against the
separately retained `67c9b23213b300da3d01d94b56cc826485aa81b6` worktree/target,
with zero PASS-to-FAIL transitions. Run rustfmt, strict workspace/all-target
Clippy, diff checks, and the WASM core compile check; compilation is not
browser behavior. Preserve and report any new first difference rather
than expanding into the deferred injector or another owner.
