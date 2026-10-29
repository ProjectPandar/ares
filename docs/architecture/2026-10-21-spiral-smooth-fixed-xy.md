# ARD: Smooth Spiral XY serialization matches GCodeLine::set

Status: SPEC approved; Native-first RED authorized, GREEN requires separate
review approval. Base89560ca66ff8418bcf175d573df299955f9a216f. Independent
branch, NOT stacked on the spiral-overlap transition chain.

## Upstream owner and exact Rust destination

OrcaSlicer2.4.2:
- `GCode/SpiralVase.cpp:182–183`, SpiralVase::process_layer smooth branch
  calls line.set(X,target.x) / line.set(Y,target.y), with float targets;
- `GCodeReader.hpp:57`, GCodeReader::GCodeLine::set accepts float and
  defaults decimal_digits=3;
- `GCodeReader.cpp:323–326`, std::fixed << std::setprecision(decimal_digits)
  writes that FLOAT value, retaining trailing zeros; raw-word replacement
  continues through357.

Rust destination only
`crates/ares-core/src/gcode_spiral_vase/project.rs::format_axis:340–342`,
called by project/smoothing.rs::smooth_line:55–56. Current trim_fixed
manually rounds a f64 then strips fractional zeros/point. That is an
inaccurate compatibility shell for the cited setter, not an Ares output
policy to preserve.

Future included rewrite: fixed-three-decimal serialization, semantically
like existing format_z, with the value at the Native FLOAT/f32 parameter
boundary BEFORE formatting. Do not multiply/round a f64 then print it;
no extra pre-rounding, epsilon, decimal-string conversion or double-round.
Tie handling must agree with C++ fixed formatting (including exact f32
halfway values and both signs); do not use f64::round/ties-away as a
surrogate. Verify the relevant boundary rather than assume a textual
format specification alone guarantees identical numeric semantics.

If trim_fixed becomes orphaned, delete that local unused helper. It has
no other project consumer. Do not retain a legacy fallback or introduce a
new generic formatter/configuration layer. format_z/format_e/general
GCodeWriter remain unchanged. No new Ares pipeline behavior.

## Native-first ordinary GUI pair

PrintConfig.cpp5837–5842 coBool spiral_mode_smooth, simple GUI label
Smooth Spiral. Closed8-sided cylinder radius6mm/height1.2mm,18vertices/
32triangles, six layers. Both projects use spiral_mode1, Classic/one wall,
0% sparse/no top-bottom shells,0.2mm layers, aligned/staggerOFF/scarfNONE/
wipeOFF/no brim/skirt/arcOFF/relativeE. Both layer-retraction settings0.

Actual official CLI-exported INPUTS, not post-export edited snapshots:

| Project | SHA256 |
|---|---|
| control smooth0 |61e8e63712918c1c167d4276eb73cbd2881d9a76054f42c3b9a91ac6c058be26|
| positive smooth1 |a48e157624f34a7e7425e95dddafaeb7676692dc23674a03eac41a09efb2356e|

Only ZIP payload Metadata/project_settings.config differs; only JSON key
spiral_mode_smooth0→1. Native original/repeat/seeded normalized outputs
agree per case: control completePASS, positive33 line differences.
First strict discrepancy non-M73 at124:

```
Native: G1 Z0.400 X105.930 Y105.930 E0.00112
Ares:   G1 Z0.400 X105.93 Y105.93 E0.00112
```

This is TEXTUAL parity only, not different physical XY/E/Z or material.
All33 differing command numeric values agree in diagnostic comparison;
acceptance remains strict ordered bytes, no numeric/motion normalization.
No M73, header, E, timing, configuration or trailing-line mask.

## Read-only proof and limits

Evidence `/home/indexyz/ares-coord/evidence/w5-spiral-smooth-fixed-xy/`
contains actual exports, per-ZIP hashes, launches/full outputs, matching
DWARF debugger scripts/raw snapshots and SHA256 manifest.

80 Native X/Y setter calls from SpiralVase, all decimal_digits3;80 Rust
format_axis calls from smooth_line. The first values align at the Native
f32 boundary and the source owns the fixed-vs-trimmed difference. Final
valid GDB whole outputs equal respective non-debug outputs; reads and
breakpoints only, no inferior call/memory/register write.

Do NOT claim all numeric inputs are bit-identical.9/80 later effective
f32 values differ; Rust Point2::new currently quantizes through round_6.
All80 captured inputs still yield the same three-decimal numeric text in
this witness. Those nine low-bit differences require independent numeric
witnesses and are explicitly DEFERRED: do not change projection, Point2,
interpolation, distance gates or E ratios to fix trailing zeros.

The six-layer entry/final states do not overlap. The accepted overlap
chain8afae25af is separate, frozen and not consumed by this branch.

## Included / deferred

Included after GREEN approval: source-faithful f32-boundary fixed3 XY
serialization and deletion of only newly orphaned local trim helper.
No new public API/dependency/platform I/O or Rust module over400LOC.

Deferred/excluded: nine low-bit input differences and geometry arithmetic;
Point2/global quantization; smoothing projection/resampling/short-move or
flow math; Z/E/general writer formatting; processor/cache flags; M73 or
output-normalizer changes. Also exclude all overlap transition state/
branch/passthrough/header behavior, W2 brim/skirt, W1 painted/MMS, W6
medial/interlocking, W3 combine-solid clearance and W4 stack25 scope.
No coordinate-, value-, shape- or layer-specific special case/fallback.

## Formal RED then review before GREEN

Freeze the two actual exported3MFs as INPUTS only with SHA/per-entry
manifest and provenance. Two tests DIRECTLY call existing main
native_stream_case::check for Native double replay and complete validated
public API/bare CLI comparisons. No copied runner/manual normalization,
JSON/replay logic, private seam, extra Native content assertion, ignored
or expected-failure test; SHA freezes settings and full comparison is
oracle. Preserve full artifacts and both public attempts on RED.

Official/seeded RED: controlPASS/positiveactualFAIL, stable Native replay.
Send REVIEW-REQUEST and STOP awaiting explicit GREEN approval. Future
GREEN must keep all tests/input/helpers frozen, prove strict equality and
C++-compatible f32 fixed/tie behavior without widening numeric scope.
No implementation or GREEN sufficiency is claimed here. FULL only under
an explicit later window; no discovery cache or GDB proof is formal
public-API or exact-main integration evidence.
