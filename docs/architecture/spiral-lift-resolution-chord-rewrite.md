# Spiral lift: resolution-chord rewrite boundary

Status: literal writer port implemented;
complete output remains different at residual Gap infill direction. No
spiral, FillBase or whole-family closure is claimed.

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
that scope requires an explicit owning source/config boundary.
The active ordinary formatter must remain exact; no whole-method closure
is inferred from the default-precision witnesses alone.

## One ArcWelder kernel, shared ownership

The unchanged ArcWelder transcription now lives at
`geometry/arc_welder.rs::arc_discretization_steps`, with crate-private
visibility. Processor arc targets and `processor/arc_accounting.rs` import
it directly. The move `a68124fa557bde399d42b9eb2120a77f332c4ca5` has no
semantic delta. Body/doc identity and the existing processor byte-output
assertions were preserved; no second kernel or compatibility re-export.

The writer must use this one helper. Keep its8-128 clamp outside the
generic kernel and preserve processor operation order, EPSILON branch and
callers' existing tolerance values. No global exp/pow/arc-math rewrite or
numeric bias. The relocation is not a writer behavior change.

Reuse ordinary writer axis formatting. A shared writer formatter change
rather than an as-is call needs its own boundary.
Do not touch the separate Klipper helper/callers at
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

This slice builds on the Klipper prerequisite; do not independently fix the
prerequisite. Only then qualify public Native/API/CLI RED cases, using fresh ordinary
Native double stability and the validated public API plus CLI complete
exports. Register new tests through `option_coverage.rs`, not the root
harness, and preserve all retired registrations.

The full ordered comparison stays strict with only the existing canonical
generator-line normalization. M73 may first differ because total time
reflects later geometry; an external diagnostic may identify the first
non-M73/motion difference without changing the test. It must be this
spiral-lift boundary after the Klipper prerequisite, not an ignored earlier
unrelated difference. If another earlier non-M73 boundary appears, attribute
it before porting. No silent extra record filter or input variant.

GREEN must pass complete output, M73 included. If the literal spiral port
moves the first difference to the separate FillBase residual gap
or another producer, retain that full failure and attribute it to its
owning slice. Do not call the slice GREEN, mask later records, broaden the
spiral commit into FillBase, or claim a whole-file/family closure from a
spiral-only diagnostic.

## Qualified public RED on the Klipper prerequisite

The Klipper prerequisite is `9add9e077149d4f522f909b3fd3712803ebda9f1`.

One ordinary selected run exercised the two frozen public cases, each
with two fresh stable Native exports, one validated public API call and
one CLI call. Both strict complete-output tests fail as RED, with M73
fully retained, and API/CLI are raw-byte equal. The complete-output first
differences remain velocity L19 `M73 P0 R8` versus `M73 P0 R7`, and
positive-SYNC L46's earlier API `M73 P26 R5` placement.

In an external M73-excluded diagnostic, the first
remaining difference is the actual spiral in both: Native
`G1 X106.695 Y112.876 Z.417` versus API/CLI
`G1 X107.217 Y113.411 Z0.457143`. Velocity original lines are163/162;
positive-SYNC is169/169. The ordered prefix before this spiral agrees in
that diagnostic; no Klipper field or other record is additionally removed.
The tests and shared strict comparator do not use this filter or a fixed
coordinate/count oracle. No writer port or GREEN is part of qualification.

Full GREEN still includes M73 and every other record; an endpoint-only diagnostic cannot substitute for it.

## Arc-enabled public RED qualification

Three further unchanged inputs exercise the genuine G17/G3 branch:

- sequential SHA256
  `baa6c8d352d5e62c400cc450d50de643ce868fcc619a71352748f89aabe7aab1`;
- stock SHA256
  `b3f72543f8794281e1d99ef784a993dbb0523bfbe96468403cec4b21da34a5a2`;
- fuzzy stock-control SHA256
  `5d4a5f95fd4c12244e87b17e9e64052f56d66fd83df405438fb4c4e34c8c524c`.

Their separate public module is `spiral_lift_arc_enabled.rs`; the earlier
two tests and fixtures are unchanged and were not rerun. Each added case
has two fresh stable Native exports, one API call and one CLI call, with
raw API/CLI equality. All three strict complete-output tests remain RED:
API inserts an earlier M73 at original line110. No timing record is masked
or removed by their comparator.

The first external non-M73 differences are sequential327/327 and
stock/fuzzy322/321: Native `G17`, then
`G3 Z.8 I1.151 J.396 P1  F30000`, versus API/CLI `G1 F30000`, then
linearized XYZ points. This directly qualifies
`GCodeWriter.cpp:1262-1274`, not a formatter-only or later gap witness.
The two disabled-arc cases qualify the resolution-chord segment branch
at1227-1259. Both branches remain inside the writer slice.
Across the two selected runs: five expected strict failures, Native10,
API5 and CLI5, with no retry, observer, writer port or family claim.

A separate E5 check is deferred until after the FillBase gap-direction
port: rerun its complete output and test whether the same-layer feed
mismatch disappears. If it does, no cooling slice is needed; if it remains,
it needs a separate attribution for `CoolingBuffer.cpp:989-1020`. Neither
cooling nor FillBase is folded into the spiral writer commit.

## Literal port execution and residual boundary

The configuration fix is `e015de3bedcf200feae6e70b0a05cce7a3e23045`.

The writer now uses the single neutral ArcWelder helper for positive
resolution, clamps8-128 locally and selects128 for zero resolution. It
reuses the ordinary writer XYZ/IJ/F formatters unchanged, emits the optional
comment on the feed line, interpolates current writer Z, and emits the exact
final XY/Z point. Both branches use the existing Z-speed selection helper;
the emitted feed is retained in motion state instead of overwritten by the
callers' unconditional travel-speed assignment. No new configuration rule,
shared formatter edit, kernel copy, precision waiver or legacy fallback.

One fixed 15-test inventory ran once before and once afterward: the five
complete public witnesses, seven existing lift-family checks and three
public configuration controls. Both runs reported5PASS/10FAIL/0ignored,
with zero status flips or prior-PASS to FAIL; each retained19 Native
command-status records. The two by-object controls and all three config
controls stayed passing. Three firmware controls retain their complete
M73 failures. The Auto-lift query-frame and two-nozzle controls already
failed Native witness premises before the port and still do; they do not
establish family parity and their assertions were not weakened.

All five target cases have stable Native replays and raw API/CLI equality.
After the literal port, their external-only first non-M73 difference is a
reversed Gap infill entry: velocity original lines436/435, positive-SYNC
446/445, sequential920/920, stock/fuzzy915/916. The ordered diagnostic
prefix includes the matching spiral; no additional record class is removed.
The strict tests still fail with M73 retained, so this is **NON-GREEN**,
not acceptance of a coordinate-only or filtered prefix. No FillBase edit
is part of the writer commit.

Fmt and strict workspace/all-targets Clippy pass.

## Breadth, failure inventory and exit criteria

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
processor consumers unchanged by helper relocation and fmt/strict Clippy.
Docs-only preparation is not RED/GREEN qualification.
Core logic stays an API on browser WASM, Windows, macOS and Linux, without
filesystem/UI/terminal behavior, a new crate or an independent Ares pipeline.
