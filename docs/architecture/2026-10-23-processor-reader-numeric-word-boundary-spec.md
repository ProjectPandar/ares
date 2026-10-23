# SPEC proposal: processor GCodeReader numeric-word boundary parity

## Authority and exact base

Docs-only ONE new SPEC on exact fresh main
**9cfaf0d16f0f5c4bf5fb6bf6c62d1c54e0d94ecf**. No tests,fixtures or Rust changes
are included. Discovery producers ran before this document commit against
that clean exact base,CPU12. Request separate test-only RED authorization;
this SPEC does not authorize production GREEN,FULL or integration.

## Owning upstream rewrite slice

Pinned OrcaSlicer **8500fcdccaa10b5099ac20d252af3a7c560046f1**:

- `src/libslic3r/GCodeReader.cpp:29–84`,
  `GCodeReader::parse_line_internal`:walk command/argument words;parse numeric
  prefix with fast_float;accept axis value only if the end pointer satisfies
  `is_end_of_word`;otherwise `skip_word` discards the remaining whole word.
- `GCodeReader.hpp:183–196`:ASCII space/tab,semicolon/CR/LF/NUL delimit words;
  the next uppercase axis letter is **not** a word delimiter.
- `GCode/GCodeProcessor.cpp:3813–3822`,
  `GCodeProcessor::process_G1`:only `line.has_x/y/z/e/f` values become optional
  coordinate/feedrate inputs. Numeric prefix alone does not establish presence.
- `GCode/GCodeProcessor.cpp:3868–3891,3965–4080`:downstream zero-motion/time
  accounting consumes those parsed values. This proposal does not rewrite the
  planner/cache/exporter to compensate for wrong lexical input.

Rust destination:the existing `word` helper in
`crates/ares-core/src/project_slice/gcode_emit/processor/motion_util.rs:6–18`,
consumed by `MotionState::motion` in `processor/motion.rs:252–280`.
Current helper finds a letter anywhere,ends numeric text at the next alphabetic
character and treats it as a valid word. Thus it accepts X/Y/F from
`G1 X101Y100F6000`,while the source reader rejects that entire joined axis word.
The helper is temporary incomplete scaffolding for this upstream reader
contract,not an Ares-owned permissive G-code grammar to preserve.

## Real Native-backed witness on exact main

Origin:already official-exported INPUT
`tests/parity/fan-mixed-timing/control.3mf`,SHA
`b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298`.
Stock named printer **Creality Ender-3 0.4 nozzle**,process
**0.20mm Standard @Creality Ender3 0.4**,filament **Creality Generic PLA**;
all three names exist in official2.4.2 shipped resource catalog and their JSON
hashes are recorded. Existing32-sided12×6mm cylinder,30layers,0.2mm,Classic2walls,
15%crosshatch,3top/bottom,PLA220C;not a synthetic preset or nozzle substitution.
Existing export provenance retained in source fixture manifest;new witnesses
are GUI-reachable edits of that stock-origin project,not falsely described as
new GUI-export transactions or untouched factory presets.

GUI Machine G-code → **Before layer change G-code** is an ordinary multiline
field (`Tab.cpp:4629–4638`;`PrintConfig.cpp:1147–1152`,coString). Keep original
header/comment/expression/G92E0,append the same safe-bed absolute positioning
anchor,then vary only whitespace in its following movement:

```
;BEFORE_LAYER_CHANGE
;[layer_z]
G92 E0
G1 X100 Y100 F6000
G1 X101 Y100 F6000
```

Target changes only the last line to `G1 X101Y100F6000`. No temperature/startup
fan edit,E extrusion,new expression,dwell,geometry,filament or process variant.
Physical compact G-code is a GUI-reachable input;**upstream's analysis reader
rejects the joined word**. Matching that source behavior is the goal,not
claiming a firmware execution rule or rejecting the project at admission.
Both fan_speedup_time and fan_kickstart remain0,so FanMover is not the owner.

Final pair external to repository until RED approval:

| Input | SHA256 |
|---|---|
|spaced control|3a9c71fea578cca734cc03236cfbfc3cc1c664ef33b7b4d275a0d7ef76c3a7d9|
|joined target|14a8c16229dad5db584b4b15573db49c37aadc2c0b804836047b50acddf6cb54|

Only `Metadata/project_settings.config` changes from origin;only
`before_layer_change_gcode` differs,including between pair members. All other
config and ZIP payloads/geometry/placement are unchanged. Repacking bytes/hashes
are frozen;no Native-produced expected G-code is checked into the repository.

Official and seeded0 **both**:control complete ordered PASS,target real complete
ordered FAIL,after successful stable Native double replay then successful
public validated API and actual CLI0. Raw API==CLI in each case and across
modes. First normalized difference **byte1459,line68**:
Native `M73 P14 R5` versus Ares `;TYPE:Inner wall`;Native119441bytes,
API/CLI119430bytes. This is not a runtime refusal,missing file or Native-version
exception. Progress relocation/remaining-time differences are active outputs,
not inert config round trips. Diagnostic SequenceMatcher hunks are retained
only for localization;they are not used to mask/sort/accept the full streams.

Discovery root:
`/home/indexyz/ares-evidence/w3-gcode-stock-discovery-9cfaf0d16/`.
Final witness subroot `compact-motion-no-fan/`;results/commands/statuses/full
Native/API/CLI streams,exact build CargoJSON and API rlib provenance retained.
Original date/generator-only normalization used,retaining all M73/time/fan/E/F/
wipe/start/end/config/footer bytes. Independent script audits do not pretend
formal shared-checker assertions have already been added/executed.

## Bounded investigation history,not additional SPEC proposals

One prior kickstart0→0.5 probe on same stock origin was wholePASS in both modes:
**bounded null**,not a defect/universal kickstart claim. Its full evidence stays
in `kickstart/`. Initial joined-word probe with common leadtime1 also had this
progress difference;removing leadtime to0 preserves it. That bounded reduction
is in `compact-motion/` and final `compact-motion-no-fan/`,not a second proposed
FanMover change. No parameter search,trial patch,debugger/Native rebuild or
production instrumentation was used. Across all three probes:24actual Native
slices,12validated API,12actual CLI,all runtime0 on CPU12;all repeated Native
refs stable. Main Rust binaries built exactly9cf,not reused from older main.

## Included / deferred / nonnegotiable boundaries

Included future rewrite:port numeric-word acceptance/end-boundary and
skip-whole-word behavior **as consumed by processor coordinate/feedrate
parsing**,with existing numeric storage/promotion/call ordering intact.
Destination starts with `processor/motion_util.rs::word`. Keep changes source
literal;do not merely special-case the witness string or invent an independent
validator/parser framework. Audit existing helper consumers before GREEN;
if parity requires edits in frozen planner/cache/firmware or machine paths,
STOP and request a scope amendment rather than silently expanding.

Deferred:general complete GCodeReader rewrite,lowercase/duplicate-word/unknown
axis grammar,compact command opcodes,exponent/NaN/locale/new numerical behavior,
Klipper assignment parsing,firmware E semantics,arcs/kinematics,progress-cache
algorithms,fan/kickstart/startup/temperature policies,geometry/admission and
all unrelated templates. Existing helper's other source semantics are not an
excuse to change downstream algorithms or add fallback/rejection-as-success.

Explicit exclusions:W1 startupfan/machine.rs;W4 machine/temperature.rs scanner;
W5 wipe tower/brim;W2 frozen firmware literal-E/VG1 cache and other chains;W6
frozen seam/ironing/sanitizer/defaultfloat/loop chains. Exact audited W2 range
47821c760..515e959759f1357b34765fcb7fbe08e9b1630202 and W6
ad5111f0b..26a87adfe0dfc04dbd4a79a652eb015a7194b385 have no destination-path
change/history and no numeric-word-boundary SPEC ownership hits. Adjacent
processor path ownership is acknowledged,not inferred safe from file absence
alone;their documented goals/changed scopes are retained in ownership audit.

## Next staged gates,only after approval

1. Freeze exact proved input bytes/SHA in test-only RED;two ordinary unchanged
   `native_stream_case::check` calls `(false,|_,_|{})`,complete ordered equality.
   Reproduce dual1controlPASS1genuine targetFAIL,not inverted/ignored tests.
2. Send RED-RR before production. Keep full Native2-first success/stability,
   validated API/actual CLI success,raw equality,unchanged comparator. No M73,
   timing,E/F or config masks/goldens/tolerances/private-state assertions.
3. Only separately authorized GREEN ports the reader lexical boundary,then
   strict pair/control must both PASS without touching downstream policies.
   Unexpected first-owner/residual →STOP. fmt/Clippy/nextest gates and any FULL
   or integration need their own directed authority/exact base.

No tests or implementation accompany this docs-only SPEC;actual difference
is already proven at exact main,not a static hypothesis or scaffold feature.
