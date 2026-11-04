# ARD: Port GCodeReader float XY distance to the spiral reader

Docs-only SPEC; Native-first RED follows. Independent of the
spiral overlap8afae25af and fixed-XYb4d5eade7 chains.

## Source owner and non-negotiable rewrite boundary

OrcaSlicer2.4.2 `libslic3r/GCodeReader.hpp:47–51`,
GCodeReader::GCodeLine::dist_XY: FLOAT coordinate operands, subtract to
float x/y, float x*x+y*y, sqrt returned as float. Reader axes and parsed
line axes are float, not decimal f64. This arithmetic order is observable.

`GCode/SpiralVase.cpp:84–96` consumes that distance for float
 total_layer_length;143–147 float dist/len/factor;157–159 uses progress
for the finishing E factor and fixed5 product. This is an upstream
reader-distance port slice, not a new Ares path/flow design.

Rust destination ONLY
`crates/ares-core/src/gcode_spiral_vase/project.rs::Movement::from`'s
xy_distance expression (currently around245). The scaffold currently
subtracts decimal f64 coordinates and calls f64 hypot, casting the
result later during float accumulation. That does NOT reproduce the
owning source boundary.

Future GREEN must narrow EACH coordinate operand to f32
BEFORE subtraction, compute f32 x*x+y*y, then sqrt with f32 result.
Do not use hypot, f64 subtraction then narrow, wider products/sum,
manual rounding, epsilon or progress snapping. Existing wider
xy_distance field may carry the f32 result widened exactly to f64;
no field/API redesign is needed. Use the existing shared distance
calculation for total pass and emission pass, not duplicated algorithms.

## Ordinary GUI, single-key frozen exported input pair

GUI Spiral finishing flow ratio, PrintConfig.cpp5868–5877 coFloat0..1.
Closed24sided tapered cylinder radius6→5.4mm, height1.2mm,50vertices/
96triangles, six layers. Both spiral1/smooth0, Classic/one wall/no sparse,
no top/bottom shells,0.2mm layers, aligned/staggerOFF/scarfNONE/wipeOFF/
arcOFF/relativeE, no brim/skirt. Entry second layer, final sixth:
NO overlapping transition states and NO Smooth XY formatter call.

Actual official-exported INPUT3MFs (not post-export edited ZIPs):

| Input | Whole SHA256 |
|---|---|
| control finish1 |833b90212289240a2bb655394129d054726fcb8877842f6cdc1264e06863ef7b|
| positive finish0 |58483c5d001e4d491f1a40e4db76a99e41933fad9c3ee9bd6da607dea2a4d196|

Identical ZIP inventory, only changed payload project_settings.config,
only changed JSON key spiral_finishing_flow_ratio1→0. Whole and per-entry
hashes/provenance will freeze these inputs, not any expected output.

Official1/official2/seeded discovery: control completePASS all3,
positive exactly1 difference all3. First strict non-M73 at335:

```
Native: G1 X113.693 Y113.693 E0.02262
Ares:   G1 X113.693 Y113.693 E0.02261
```

Only existing canonical generated-by identity/date normalization allowed;
no M73/time/header/E/XY/config/trailing-line filtering or tolerances.

## Measured localization, not a formatting or state hypothesis

Read-only GDB captured51 actual E setter arguments from each slicer.
Raw float/f64 bytes avoid lossy printed-value reconstruction. At index38,
identical incoming E.04523 becomes:

| Value | Native | Rust |
|---|---|---|
| progress |0.5|0.5000000596046448|
| finishing factor |0.5|0.4999999403953552|
| E product |0.02261500060558319|0.02261499874293804|

The products already differ before fixed5 formatting; both formatters
correctly render opposite sides of the decimal boundary. No E formatter
change is justified. Flags/chunk counts agree, with no entry/final overlap.

External source-arithmetic replay of matching6 captured input chunks and
initial reader positions reproduces ALL51 actual Native E products and
Native progress values, and ALL51 actual Rust products. Locally changing
only distance arithmetic to source float on unchanged wider Rust target/
reader fields reproduces all51 Native products. This diagnostic is NOT
a production patch, public API RED or proof of whole-output GREEN.

Successful Native/Rust debugger full outputs equal ordinary CLI outputs;
no inferior calls/program memory/register writes. One failed broad
function breakpoint also matched a lambda; incomplete capture excluded,
retained transparently, then replaced by exact-source breakpoint.

## Included / deferred / exclusions

Included ONLY source-faithful f32 dist_XY arithmetic at Movement::from.
Keep target_x/target_y/xy_distance/ReaderState/ParsedLine field types,
parsing, reader updates and dist_E unchanged as temporary compatibility
shells for adjacent behavior outside this port. No other consumer's
implementation is rewritten. Existing consumers naturally receive the
source distance through the same leaf; no per-feature legacy fallback.

Deferred: parser/reader storage rewrite, E semantics, wider public API,
Smooth projection/Point2 quantization and the9 prior low-bit samples,
other consumer behavior, thresholds/resolution gates/clamp policy,
Z/E/general formatting and flow formulas. Do not claim this slice resolves
those independent numeric observations or caller cleanup.

The two spiral points remain excluded: transition_out state/
if-else priority/passthrough/header counts, and fixed3 format_axis/orphan
trim deletion. Also exclude brim/skirt, painted/MMS, medial/
interlocking and combine-solid clearance. No processor/header/
normalizer work, shape/value/symmetry special case, dependency/platform I/O
or new module. All future strict residuals require STOP/report, not scope
expansion or output masking.

## Formal RED

Two tests DIRECTLY reuse main native_stream_case::check, which double
replays Native and delegates complete validated API/bare CLI comparison
to main native_api_cli. No duplicated runner/replay/normalization/JSON,
Native content constants/assertions, expected-failure flags or snapshots.
Native-first controlPASS/positiveactualFAIL in official and seeded,
with complete streams.
Freeze actual exported3MFs plus input manifest/README. Later GREEN must
preserve tests/inputs/helpers.
