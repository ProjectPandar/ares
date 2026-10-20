# PrintObject::combine_infill: scaled-width clearance arithmetic

Status: docs-only SPEC on `worker3/combine-solid-clearance-spec`, based on
merged main `231bd11ccae4ac26304aa89772a1e5c4b2fd532c`.
COORD accepted the six-case Native discovery and source boundary and authorized
this SPEC followed by a separate RED. GREEN requires independent RED review;
no implementation or FULL is authorized by this document. Integration is
COORD-owned, queued after W2; no worker rebase/merge/push/window is implied.

## Owning upstream rewrite slice

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintObject.cpp:4175-4180`,
  `PrintObject::combine_infill`: at100% use `stInternalSolid` and configured
  `internal_solid_infill_pattern` rather than sparse internal surfaces.
- `4252-4264`: `float clearance_offset` adds `0.5f` times the retained
  perimeter Flow's `scaled_width()` and pattern-dependent `1.5f`/`0.5f`
  times the retained solid-infill Flow's `scaled_width()`. The existing
  rectilinear/monotonic/grid/lateral-lattice/line/honeycomb/lateral-honeycomb
  predicate chooses1.5f; other configured patterns choose0.5f.
- `src/libslic3r/Flow.hpp:61-62`: width is float;
  `scaled_width()` returns `coord_t(scale_(m_width))`, truncating the scaled
  width **before** multiplication. `libslic3r.h:93` scales by division.
  Ordinary C++ arithmetic converts these coord_t operands to float for the
  float multiplications and addition; do not replace that order with f64 math.
- `PrintObject.cpp:4265-4291`: offset/intersection subtraction and combined
  surface transport consume this clearance. They are adjacent, deferred
  behavior, not permission to port kernels or rewrite surface algorithms.

Rust destination is exclusively the clearance calculation in
`crates/ares-core/src/project_slice/prepare_infill/combine_infill/process.rs`
`combine_group`, currently205-208. It presently sums raw retained f32 widths
in f64 millimeters, divides the combined value by `scale.factor()`, then
casts the final delta to f32. Replace only this arithmetic with the source
order: existing `Flow::scaled_width(scale)` integer truncation, conversion
to f32, f32 multiplication and addition. The existing helper in
`project_slice/perimeters/types.rs:17-18` already implements the truncation;
do not change Flow construction, transport or that helper.

The existing `combine_group` record/geometry machinery is the compatibility
shell around this upstream operation, not an Ares-owned infill design.
This slice replaces its mismatched arithmetic, not the surrounding pipeline.

## First inconsistent input, observed before offset

Single-region authored75-layer overhang, same geometry/placement/profile,
layer0.2 and nozzle0.4. Existing canonical binaries/DWARF and read-only GDB,
no Native source instrumentation or rebuild:

| Configured solid pattern | Native actual delta | Rust actual delta |
|---|---|---|
| monotonic,1.5f branch |899998.0|900000.0|
| concentric,0.5f branch |449999.0|450000.0|

For each pattern the first8 pre-offset intersection contours and holes match
exactly, including coordinate order. Rust local tops are2,4,...16/count2;
Native's matching calls use Miter and both miter limits are3.0. Thus the first
observed different offset input is the clearance scalar, not its geometry.
Successful diagnostic Native complete streams match canonical references
under generator/date identity normalization; CLI diagnostic streams are
raw-identical to canonical CLI. Rectilinear shares the cited1.5f source
branch; its strict witness is independently recorded, not a claimed GDB run.

Ares retained widths are observed as0.44999998807907104. Source integer
scaling truncates that value to449999; applying the two float factors gives
899998 or449999, agreeing with the actual Native arguments. The current
late-rounding calculation yields900000 or450000. This source consistency
calculation diagnoses the mismatch, not a replacement acceptance oracle.
Direct Native-width probes without captures are not claimed as width
provenance. Failed pointer-typedef decoders are preserved/excluded; the
accepted reader strips typedefs and verifies Point16-byte layout.

## Frozen Native matrix and public seam

Reuse `tests/parity/combine-infill-region-view/combined-single.3mf` directly
for sparse15-on: its discovery SHA is identical, not a new baseline ZIP.
Freeze only the five other existing discovery inputs, without regenerating
geometry or changing non-settings ZIP members. Only GUI density, combination,
internal-solid pattern and maximum combination height change.

| Case | Density / combination / solid pattern / maximum | Both modes |
|---|---|---|
|sparse15-on|15% /1 /monotonic /100%|PASS control|
|solid100-off|100% /0 /monotonic /100%|PASS control|
|solid100-on|100% /1 /monotonic /100%|DIVERGENT target|
|solid100-rect|100% /1 /rectilinear /100%|DIVERGENT sibling|
|solid100-concentric|100% /1 /concentric /100%|DIVERGENT sibling|
|solid100-one-layer|100% /1 /monotonic /0.2|PASS control|

Discovery has24 fresh Native streams and12 CLI outputs, two Native repeats
per input/mode, stable repeat/cross-mode identities. Strict first differences:

| Target | Byte / line | Native | CLI | Lengths |
|---|---|---|---|---|
|monotonic|3555 /158|M73 P5 R15|G1 X113.987 Y111.832 E.01673|199325/199271|
|rectilinear|3584 /159|M73 P5 R15|G1 X108.168 Y106.013 E.2555|199729/199675|
|concentric|11014 /487|G1 F2154|G1 F2160|165913/165913|

M73/feedrate are downstream witnesses, not repair sites. First changed
monotonic/rectilinear motion is physical51/Internal solid; concentric's first
changed motion is physical5/Inner wall feedrate. Do not filter these effects.

The confirmed test seam is fresh canonical Native versus public byte-in/
options-to-byte-output API and CLI, not private numerical unit tests. RED must
reuse main `NativeCase` or the existing W3 `case_runner`/`complete` lifecycle;
no copied runner, second normalizer or assertion policy. Reusing W3 preserves
both fresh repeats and their producer records before complete assertions.
Admission asserts one model part/no modifier and GUI values; effective
Native footer and positive solid/sparse activity demonstrate active inputs.
No previous assertion, case/input or helper behavior may be weakened.

## Non-negotiable decisions and stage exits

1. SPEC commit is documentation only, parented on231bd11cc. Per COORD's
   discovery approval, RED may follow as a separate commit; no production
   edit before its independent review.
2. RED freezes the five existing target/control ZIPs and reuses the identical
   sparse control. Register six complete-output cases through shared lifecycle.
   Require3 intended strict FAIL/3 PASS per official/seeded mode, all prior44
   selected cases PASS, repeat stability/API-CLI raw equality and retained
   existing Native/API/CLI hashes/assertions. Preserve command/status/binary/
   input/output receipt links and repeatable artifacts; report RED to COORD.
3. Only after RED approval, GREEN changes **process.rs clearance arithmetic
   only**, using truncated Flow scaled_widths and float operations in upstream
   order. Freeze SPEC/RED/helpers/inputs. Keep the current pattern predicate,
   per-region views, actual layer-id0 skip, height cap/count/area thresholds,
   intersection and surface rewrite/transport, Flow values/keys unchanged.
4. Any strict residual means **STOP/report**. Explicitly defer offset kernels,
   batching/per-component union/miter/topology, phase/graph/bbox, bridge angles,
   max-height arithmetic, Flow grouping fields, E/wipe/retraction/feedrate/
   cooling/time/progress. No adjacent port or unsupported-feature guard.
5. Gates: `cargo fmt --all --check`; locked workspace/all-target Clippy with
   `-D warnings`; locked core+wasm browser wasm32 check. CPU12 selected work,
   `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0`; no FULL until COORD releases the
   after-W2 integration window. Later acceptance needs then-current main,
   preserved per-stage range-diff and exact clean final-hash fresh official+
   seeded FULL with zero FAIL/skipped; refresh if main advances.

Core remains platform-neutral for browser WASM, Windows, macOS and Linux.
No new API/configuration, filesystem/terminal/UI/OpenGL behavior, fallback,
feature flag, new allow/expect/ignore, tolerance, inverted assertion, Native
G-code golden or output filtering. Only existing generator/date identity
normalization; all M73/time/feedrate/E/wipe/retraction/footer bytes significant.

Accepted discovery and producer/localization audits:
`/home/indexyz/ares-evidence/w3-combine-solid-readonly-231bd11cc/README.md`.
The prior minimum-area7/7 PASS and combine-height6/6 PASS matrices remain
bounded negative evidence; they do not authorize any adjacent change.
