# ARD: Firmware retract simulation forwards literal E words

Docs-only SPEC; formal Native-first RED follows. Independent of the
spiral chain. No production change in this phase.

## Upstream owner and non-negotiable Rust boundary

OrcaSlicer2.4.2 `libslic3r/GCode/GCodeProcessor.cpp:4873–4890`,
process_G10/process_G11 construct fake GCodeLines with literal
E=-retraction_length / E=retraction_length+restart_extra and configured
F, decrement G1 id, then delegate to process_G1. They do NOT add the
current E position when in absolute mode. The normal processor mode/
origin interpretation owns the resulting position and delta.

Thus under M82/G90, the upstream fake E is an ABSOLUTE position, even
though physical firmware retract is commonly described as a relative
amount. Port upstream simulator behavior, not a redesigned physical
firmware model. This is a processor rewrite slice, not an Ares pipeline.

Rust destination ONLY
`crates/ares-core/src/project_slice/gcode_emit/processor/motion/firmware.rs`
`emulate_firmware_retraction`'s E-word construction. Existing !e_relative
branch pre-adds state.e_position to the configured E word. That extra
reinterpretation is an incorrect compatibility shell around process_G1.

Future GREEN must pass the literal configured E into the
existing fake G1 in either E mode and let existing MotionState::motion
interpret it normally. No new mode/origin handling, alternate estimator,
legacy fallback, feature flag or output-mask workaround. Rename local
E-delta terminology only if necessary for the literal-word meaning.
Feedrates, block-kind handling and all other state stay unchanged.

## GUI-valid input-only single-key witness

Ordinary printer GUI Use firmware retraction checkbox0→1, with both
projects in absoluteE (use_relative_e_distances0). Same closed8-sided
cylinder radius6mm/height1.2mm,18vertices/32triangles, six layers,
Classic/one wall/no infill/top/bottom/brim/skirt, aligned/staggerOFF/
scarfNONE/wipeOFF/arcOFF/spiralOFF, Marlin, retract_length4,
retract_speed60/deretract40, layer-change retraction enabled.

Both use the SAME valid GUI-editable before-layer macro without manually
injected G92 E0. Native rejects that manual command in absolute mode;
earlier invalid input is excluded. Native's own generated G92s remain
untouched. Actual official-exported3MFs copied unchanged:

| Input | Whole SHA256 |
|---|---|
| control firmware0 |b5978da3842bd1734fcc36930d788c180b0ce0461605975ef9a4960c3dd79216|
| positive firmware1 |dd75e19b01ae72b130de91421e4466ba9488e657dedd4d5fe96efba6cb712f14|

Identical ZIP inventory, only changed payload project_settings.config,
only changed JSON key use_firmware_retraction0→1. Freeze actual INPUTS
with whole/per-entry hashes/provenance, not expected-output snapshots.

Official1/official2/seeded discovery: control completePASS all3;
positive10 strict line differences (five M73 insertion swaps) all3.
First strict difference at73: Native G1 vs Ares M73. This is explicitly
M73-ONLY, not a physical movement/output-E/material discrepancy, and is
accepted because it follows the processor's different fake E semantics.
No claim this is non-M73. Never filter M73 to turn it into PASS.

## Direct producer localization and limits

Read-only GDB captured13 actual Native E-setter arguments with caller
processor pre-state/modes, and13 actual Rust fake-G1 inputs with pre/post
E state. All13 pre-E/origin/absolute-mode states align. First discrepancy
at index2, second G10: preE1.0848499536514282/origin0 in both; Native
supplies literal-4, Rust supplies-2.9151500463485718. Six G10 literal
mismatches; G11 preE resets0 through existing generated G92 and both
feed4. Do not fabricate a first-unretract divergence.

Native absolute post/delta values are source-derived, NOT direct post
snapshots. Earlier zero-record Native post-G1 probes are excluded, not
proof that Native ignores G10. Final actual setter/caller capture proves
execution/argument discrepancy. Both successful debugger full outputs
match ordinary CLI outputs, no inferior calls/data/register writes.
No complete time-cache causal reconstruction or whole-GREEN sufficiency
is claimed before formal testing.

## Included / deferred / exclusions

Included ONLY firmware.rs literal-E branch semantics. Existing shared
mode/origin interpretation remains unchanged. No source change to any
consumer, cache, seam lifecycle or percentage exporter. No line-ID fix,
new private oracle or duration adjustment. A strict future residual
requires STOP/report, never expansion into those components or masking.

Deferred: setter fixed3/f32 text precision for fractional configuration,
G91/other dialects, line id, cache, viewer/role behavior, generic motion/
G92/config parsing, block classification/feedrates/time integration.
No integer-/mode-/shape-specific exception or fallback to known-wrong E.

Avoided: brim/skirt, painted/MMS, combine-solid/concentric, ranges,
medial/interlocking/SeamPlacer nearest, seam-cache/scarf flags, all
three spiral points. Their frozen chains are not modified or consumed.
No new dependencies, platform I/O, public API or independent pipeline.

## Formal RED then GREEN

Two DIRECT native_stream_case::check calls. Existing helper double
replays Native then delegates complete validated API/bare CLI comparison
to native_api_cli. No copied normalization/replay/JSON/orchestration,
Native output content constants/assertions, ignored/expected-failure
flags or G-code snapshots. Freeze only actual3MF INPUTS + input manifest/
README. Require official/seeded controlPASS/positiveactualFAIL and stable
Native repeats.

Strict ordered bytes permit ONLY existing canonical generated-by
identity/date line, no M73/time/header/E/motion/config/trailing-line mask
or numeric tolerance. Later GREEN
must freeze tests/inputs/helpers.
