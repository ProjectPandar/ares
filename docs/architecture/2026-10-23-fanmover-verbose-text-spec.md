# SPEC proposal: FanMover verbose text preservation

## Exact base and observed difference

Docs-only proposal on main
**9cfaf0d16f0f5c4bf5fb6bf6c62d1c54e0d94ecf**. No Rust or
fixture/test implementation accompanies this SPEC.

Ordinary stock-origin Ender3 project,Verbose G-code ON in both inputs,
GUI fan kick-start time **0(control)→0.5seconds(target)**:

| Input | SHA256 |
|---|---|
|control|ef1951bd9f85d84ebfaf569b5a0ac4cf729e22e3c0c0d788ea5ad71b54a6e632|
|target|42123da0f4fd38bf64a1ea498e9985640816f952b16e170fbc0a63817aa2ce7f|

Both official and seeded0:successful stable Native2 FIRST,then successful
validated public API and actual CLI0;raw API==CLI. **Whole controlPASS and
whole targetFAIL**,same normalized Native/rawpublic identities across modes.
First difference **byte6836,line208** Native `M106 S255 ; enable fan` versus
Ares `M106 S255`,Native155189bytes vs API/CLI155161bytes.

Complete strict comparison retained all fan/time/M73/E/F/wipe/custom/start/end/
config/footer bytes;only existing generator/date normalization. No comments
mask or statement that comments are non-semantic excuses the output mismatch.
Full diagnostic hunks reveal **four replacements**,all included in this one
verbose text rewrite slice (none silently deferred as a residual):

| line | Native | Ares |
|---|---|---|
|208,3285|`M106 S255 ; enable fan`|`M106 S255`|
|236|`G1 X114.099 Y113.354 E0.01706 ; perimeter`|same numeric text but `E0.01706; perimeter`|
|238|`G1 X113.763 Y113.763 E0.01755 ; perimeter`|same numeric text but `E0.01755; perimeter`|

No numerical/timing/geometry difference is claimed in this witness. These are
actual successful producer outputs,not refusal or configuration-only effects.
The diagnostic hunk inventory is not a comparator allowance or a comment-only
replacement for whole output equality.

## Owning upstream source slice

Pinned OrcaSlicer **8500fcdccaa10b5099ac20d252af3a7c560046f1**:

1. `src/libslic3r/GCode.cpp:2038`:export sets
   `GCodeWriter::full_gcode_comment = print->config().gcode_comments`.
2. `GCode/FanMover.cpp:230–236`, `FanMover::_set_fan`:delegates new fan lines
   to `GCodeWriter::set_fan`,using writer configuration rather than naked M106.
3. `GCodeWriter.cpp:1095–1130`,static `set_fan`:verbose mode appends exact
   ` ; enable fan`/` ; disable fan` before newline. FanMover.cpp:398–418 creates
   the maximal kickstart command via `_set_fan(100)`;writer comment policy
   applies to this generated command just as ordinary writer output.
4. `FanMover.cpp:74–87`, `change_axis_value`:fixed-decimal numeric rendering,
   replaces only `[pos,end)` of the raw line,where end is earliest space or
   semicolon after the axis value. Existing separator space and raw comment
   suffix are outside that replacement span and survive.
5. `FanMover.cpp:124–168,171–212`, `_put_in_middle_G1` / `_print_in_middle_G1`
   invoke that raw-axis text replacement while splitting;they do not rebuild
   tokenized whole lines and concatenate stripped comments.

This is a source-owned **FanMover verbose generated/split text** rewrite slice,
not an Ares postprocessor policy or self-designed G-code pipeline feature.
The same ordinary kickstart action generates a new fan line and rewrites a
buffered G1 segment;both must honor source text behavior for whole parity.

## Exact Rust destination and scaffold replacement

Primary destination:
`crates/ares-core/src/project_slice/gcode_emit/fan_mover/helpers.rs`:
- `set_fan_line`26–31 currently hardcodes bare `M106 S…\n`,without verbose
  policy passed by the owning export configuration;
- `replace_word`132–162 currently splits/joins all code whitespace and then
  appends comment text directly,losing the separator before `;`.

Port literal upstream behaviors,not append a witness-specific comment or
blanket `;` whitespace normalization. Numeric-axis replacement must preserve
raw bytes outside its source replacement span. Existing XY3/E5 numeric
formatting/rounding and split math remain unchanged. These helpers are temporary
incomplete shells for upstream `_set_fan`/`change_axis_value`,not independent
Ares output conventions to keep.

Only necessary caller/config handoff is included:
- `fan_mover.rs::FanMover::new` private stored verbose context;
- `layers/fan_setup.rs::gate` obtains existing
  `traversal.resolved.views.full.process.print.gcode_comments.0`;
- existing `fan_mover/fan.rs` helper callsites receive that context,without
  altering schedule,queue/threshold,kickstart duration or insertion decisions.

No new public configuration/global writer policy/helper framework/module.
Do not edit `machine.rs`,global `GCodeWriter`,template expansion or processor
parsers. Existing ordinary writer/cooling comments already match(controlPASS).
If a future patch needs more than this private text/context adapter,STOP
rather than widen into timing or other owners.

## GUI stock-origin witness provenance

Origin already official-exported INPUT
`tests/parity/fan-mixed-timing/control.3mf`,SHA
b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298.
Creality Ender-3 0.4nozzle /0.20mm Standard @Creality Ender3 0.4 /Creality Generic
PLA;ordinary32-sided12×6mm cylinder,30×0.2mm layers,Classic2walls,15%crosshatch,
3top/bottom,220CPLA. Existing official-export origin catalog names are
retained by that fixture,not fabricated synthetic preset JSON.

GUI Process→Others→G-code output→Verbose G-code,
`Tab.cpp:2753`/`PrintConfig.cpp:3965–3969`;common valueON. Printer cooling
Fan kick-start time `Tab.cpp:4557`/`PrintConfig.cpp:3849–3856`,coFloat,min0,
0deactivates,0.5is ordinary entry. Pair differs only `fan_kickstart`;both retain
fan_speedup_time0,original startup/end/before-layer templates,fan30–70%,all
geometry/config/non-settingsZIP payloads. No new custom G-code injection,
temperature/fuzzy/tower/skirt/bed/exclude-name edit or processor lexical trigger.

These are GUI-reachable edits of an official-exported stock-origin project,
not newly transacted GUI exports or unmodified factory presets. Both successful
Native projects/slices demonstrate current admission,not forced compatibility.

One earlier Mach3 flavor probe was rejected by Native as invalidvalue before
Ares;GUI source enum confirms it is commented out. Retain its1failedNative
attempt/0API/0CLI as **UNQUALIFIED_NATIVE_REFUSAL**,not parityRED or a proposed
flavor repair. No retries on that input,no synthetic enabling/Native rebuild.
The admitted verbose pair succeeded on first attempt;no numeric parameter
search,trial production patch,debugger or instrumentation.

## Nonnegotiable exclusions

Exclude Type1/tower,fuzzy/startupfan/machine.rs,temperature,
processor-word,template/bed/processor/exclude-name,skirt/brim/supportspeed.
This is FanMover's raw generated/split text,not template evaluation or processor
word tokenizing.

Known timing precision/split arithmetic is explicitly excluded here.

Also defer unadmitted Mach3/Machinekit and other flavor paths,PWM floor,
fan parsing/number grammar,speedup sign/layer chunking/relative-E/XYZ mode,
threshold/kickstart scheduling,all motion/rounding/arcs,geometry/admission.
No fixes to these adjacent behaviors are in scope for a comment-text witness.

## Staged next gates

Freeze exact proved bytes/SHA,two direct unchanged
native_stream_case checks `(false,|_,_|{})`,Native2FIRST stable→validated API/
actualCLI0/rawAPIeqCLI/unmasked ordered equality. Reproduce dualcontrolPASS/
targetgenuineFAIL in test-only RED. No inverted/ignored tests,Native
output goldens,comment masks/tolerances or private-state assertion substitutes.
GREEN ports both included literal text contracts;
all4differences must close,not just firstone. Unexpected owner/residual→STOP.
