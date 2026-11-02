# Spiral lift: resolution-chord rewrite boundary

Status: architecture ready for review; public RED waits for the approved
Klipper prerequisite. No production change or GREEN has been executed.

## Upstream owner and destinations

Source: OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/GCodeWriter.cpp::_spiral_travel_to_z:1200-1275`.
Geometry dependency: `Geometry/ArcWelder.hpp::arc_discretization_steps:48-72`.
Formatting dependency: `GCodeWriter.hpp::GCodeFormatter:320-403`,
`GCodeG1Formatter:406-417`, and `GCodeG2G3Formatter:419-430`.

Primary Rust destination:
`project_slice/gcode_emit/motion/travel/lift/spiral.rs::append`.
The current segment formula and processor-float serialization are an
incomplete compatibility shell around this writer function; replace them,
not retain them as a fallback or add an independent Ares spiral planner.

Configuration carrier:
`motion/options.rs` and `motion/options/build/fields/assemble.rs`.
The misleading field `arc_fitting_tolerance` already receives
`full.process.print.resolution.0` at assemble259. The wrong formula/clamp,
not an observed wrong configuration source, is the confirmed mismatch.
A rename to `resolution` must preserve that exact existing source, with no
second configuration path, feature flag or duplicate option transport.

## Literal writer behavior included in the active slice

- Use the current writer Z, not nominal layer Z, as `z_start`. Keep current
  coordinate ownership and XY offsets; do not add geometry compensation.
- Select `travel_speed_z` when nonzero, otherwise first-layer/normal travel
  speed, then emit its feed in mm/min. Reuse the existing
  `travel/lift.rs::z_feedrate` boundary rather than another speed policy.
- For disabled arc fitting: radius is `sqrt(i*i+j*j)`; start angle is
  `atan2(py-cy,px-cx)`. For positive slicing resolution, use the upstream
  `arc_discretization_steps(radius,2*pi,resolution)` then clamp8-128.
  For resolution0, use128 without calling the positive-deviation helper.
- Emit `i=1..segments-1` using literal `t=i/segments`, angle and linear Z
  interpolation; append the exact original XY endpoint at target Z.
  Preserve operation order and the final point; no cyclic normalization.
- G1 coordinates use the writer axis formatter, not the processor's
  six-significant-digit formatter. In the ordinary-precision witnesses,
  XYZ and F have three fractional digits with the upstream integer-domain
  rounding/trimming rules; precision is never waived in comparisons.
- The feed line carries the optional `spiral lift Z` comment in the
  disabled-arc branch; do not retain the current standalone comment line.
- Preserve the genuine arc-enabled branch: G17, CCW G3 Z/I/J/P1/F, original
  whitespace/comments, and final writer-Z update. It is an upstream branch,
  not a compatibility fallback. Apply the same literal speed selection.

The target fixtures use ordinary precision, disabled arc fitting and Auto
Lift. Upstream high-precision XYZ4/E6 and axis-remap admission/guard behavior
are adjacent boundaries: no typed high-precision carrier was found in the
current motion options, and registry presence alone does not prove active
support. This slice must not claim their parity, silently implement a
second default, or redesign global formatting/admission. Any expansion of
that scope requires an explicit owning source/config boundary and review.
The active ordinary formatter must remain exact; no whole-method closure
is inferred from the default-precision witnesses alone.

## One ArcWelder kernel, shared ownership coordination

An existing Rust transcription lives at
`gcode_emit/processor/motion/arc.rs::arc_discretization_steps:110-131`.
It implements the same upstream radius/deviation EPSILON branch and ceil
expression. Its consumers include processor arc targets and
`processor/arc_accounting.rs`. Do not copy it into the writer.

Proposed seam: relocate that implementation unchanged into a rendering-
neutral `geometry` module, with one crate-private entry reused by both
processor consumers and the spiral writer. Keep the writer's8-128 clamp
outside the generic kernel. Update both existing consumers directly and
remove the old processor-local definition; no compatibility re-export or
second implementation. Preserve processor operation order and callers'
existing tolerance values; no global exp/pow/arc-math rewrite or numeric bias.
The exact neutral module placement and edits to the processor consumers
need coordination with the shared geometry/processor owner before coding.
No relocation is part of the current documentation-only branch state.

Reuse ordinary writer axis formatting. If a shared writer formatter must
change rather than be called as-is, coordinate that diff before editing it.
Do not touch the separately owned Klipper helper/callers at
`GCodeWriter.cpp:475-481,428-471,536-583` or Rust acceleration emitters.

## Evidence and public RED gate

Start at source commit `3d11c2284553266db1113a353adc74eafcbbeeca`, tree
`520e55cc10f92bbebfa31d131824ea199c8fe289`.
The retained velocity and positive-SYNC cases have a first lifted-travel
motion difference before their later gap fill:
Native `G1 X106.695 Y112.876 Z.417` versus Ares
`G1 X107.217 Y113.411 Z0.457143`.
Their configured resolution is .012, hop .4 and arc fitting disabled.
This is saved-output/source evidence, not a fresh public spiral RED.

Frozen public input candidates, unchanged ZIP bytes:

- velocity-limit SHA256
  `d8f74deff1274800b04f021ba812c5fb84c1629e45981653218963e2cc3a1e37`;
- positive-SYNC SHA256
  `182857ce98b97df3d76968284532f516779822d2c2eb7601b0522438990b4af2`.

Wait for the explicitly approved Klipper branch/commit, then stack this
source-boundary branch on it. Do not independently fix the prerequisite.
Only then qualify public Native/API/CLI RED cases, using fresh ordinary
Native double stability and the validated public API plus CLI complete
exports. Register new tests through `option_coverage.rs`, not the root
harness, and preserve all retired registrations.

The full ordered comparison stays strict with only the existing canonical
generator-line normalization. M73 may first differ because total time
reflects later geometry; an external diagnostic may identify the first
non-M73/motion difference without changing the test. It must be this
spiral-lift boundary after the Klipper prerequisite, not an ignored earlier
unrelated difference. If another earlier non-M73 boundary appears, report
it before porting. No silent extra record filter or input variant.

GREEN must pass complete output, M73 included. If the literal spiral port
moves the first difference to the separately owned FillBase residual gap
or another producer, retain that full failure and request a coordinated
stack/ruling. Do not call the slice GREEN, mask later records, broaden the
spiral commit into FillBase, or claim a whole-file/family closure from a
spiral-only diagnostic.

## Breadth, failure inventory and exit evidence

After a qualified RED and literal port, freeze the same before/after family
inventory. Initial existing candidates: auto-lift query-frame public case,
by-object transition lift open/band, firmware zero/positive/no-hop controls,
and the reference-enabled two-nozzle auto-lift case. Explicitly select any
required reference flags; guard-returning cases are not parity PASS credit.
Include processor arc target/accounting consumers when relocating their
kernel. Final inventory, runtime budget and any source-setting edge cases
(resolution0, coarse/fine clamp bounds, nonzero Z speed, comments, genuine
arc output) must be explicit before executing them; no silent variants.

Failures to preserve: wrong input/producer, Native instability, API/CLI
failure, unrelated earlier first difference, formatter/speed/Z-state
mismatch, residual complete-output failure after spiral correction, and any
prior-PASS to FAIL family transition. No fixed Native output-count premise,
committed Native G-code, partial golden, private solver oracle, production
instrumentation, tolerance, score bias or legacy fallback.

Exit requires the public RED to become full GREEN, Native stability,
API/CLI parity, agreed family breadth with zero prior-PASS to FAIL,
processor consumers unchanged by helper relocation, fmt/strict Clippy,
and a source-cited RR. Exact runtime receipts belong to the external
evidence bundle; docs-only preparation is not RED/GREEN qualification.
Core logic stays an API on browser WASM, Windows, macOS and Linux, without
filesystem/UI/terminal behavior, a new crate or an independent Ares pipeline.
