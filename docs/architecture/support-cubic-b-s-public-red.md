# SupportCubic B-S public RED

SPEC 3d863ce63 (rebased 456fd2a2b); B-T chain 6fd0a5b3b.
B-T and B-S keep separate commits/evidence,
keeping whole-core strict gates for the real B-S caller.
No production changes belong to this RED.

## Source boundary and intended failures

PrintObject.cpp::prepare_adaptive_infill_data, especially :984–1007 and its
support_line_spacing/support_overhangs_only inputs to FillAdaptive::build_octree,
plus original bridge-candidate triangle preparation through
Tesselate.cpp:192–195. Ares destinations remain the SPEC's
fill_entities/adaptive.rs + line_spacing.rs, group_fills/params/projection.rs,
fill/adaptive/octree.rs/float_bounds.rs, and bridge_over_infill/transaction/
candidate_expansion.rs/layers.rs/types.rs. B-T geometry is a prerequisite,
not a spacing/bbox-only replacement for this complete Support boundary.

Four Support cases must be accepted by Native and both public Ares API and
CLI, retain layer structure/count, then fail the unchanged complete ordered
G-code comparator before GREEN. The scalar widths are 0.45, 0.50, 125%, auto0;
density30%, one region/count1. These cover configured width, width conversion
and automatic-width derivation while keeping original bridge payload, Support
filtering and float-frame preparation in the same closable boundary.
Controls are original A45/A50/A125/AAUTO, plus non-Support R1/L1. They must
remain strict PASS. RED does not assert a particular mismatch coordinate or
hash, normalize output, reinterpret Native rejection as regression, or change
any existing admission/line-count/layer/comparison assertion.

## Input freeze and provenance

The existing authored tests/parity/multiline-union/prism.stl is unchanged,
SHA2561d50f639d940e2a95c324b8f7ef1a33ba9bcf7e83706ba3cf0669f67f6feae7e.
The existing adaptive_cubic_precision_inputs.rs generator is unchanged: the
same vendor selection/normalization, 24 fixed options and six variable options
as the original authored matrix. Native 3MF exports and reference streams
are generated at test time by OrcaRunner, never committed. The input manifest
pins authored STL/generator bytes and per-case options; application::verify_value
continues checking every requested value on each Native-exported project.
Raw generated input digests are recorded per run, without freezing zip metadata
or substituting old output. Test labels use the existing adaptive-cubic-precision
artifact prefix; labels do not change geometry/process input.

The archived S45 input SHA8e1d3831a3db446cd62f4c381d07281520f6cc9b0b5fa3a39b295e2d0a3dcc35
is retained outside Git with its original complete outputs. Runtime regenerated
input/member differences, if any, are explained in the evidence audit rather
than hidden by reruns. Prior Support4 raw-byte invariance is provenance,
not current-main RED evidence. Current Native admission/mismatch evidence must
be collected fresh against this test-only head in both modes.

## Public comparison and evidence

Only six control! registrations are added in runner/adaptive_cubic_precision.rs:
four Support REDs and R1/L1. Existing four Adaptive registrations, generator,
API/CLI adapters, sparse witness and every strict assertion are unchanged.
Public API is slice_project_plate_validated with context seed0; CLI is actual
compiled ares slice. Both consume the same runtime exported project.
Artifacts include input.3mf, native.gcode, ares.gcode, ares.cli.gcode,
CLI status/stdout/stderr, input/reference hashes, full comparisons and first
sparse witness. All Native products remain outside Git.

Run official and seeded sequentially with a fresh artifact/cache root per mode.
Stop on non-admission or
unexpected baseline/control failures; do not weaken the matrix to obtain RED.

## RED result

Test-only RED a5f485f159c580e5fd0d9ec9c35dc67fc057d392 on main
48ff6b184b4681e30a855b0ffed42873c36d0f27.
The following evidence was collected on that exact clean test head.

Rust 1.99.0 with fresh official/seeded cache/artifact directories; fmt exit0.
Both modes ran ten selected public API+CLI tests: six strict PASS and four
expected complete-output FAIL (nextest exit100). Native export/slice commands
were20/20 exit0 per mode, with persisted reaped-child receipts. No retry,
normalization change, Native rejection, setup error or weakened assertion.

| Cases | Result in each mode | Lines Native/API/CLI | First difference |
| --- | --- | --- | --- |
| S45, SAUTO | RED | 5425 | layer4, line1418, extrusion rounding |
| S50, S125 | RED | 5353 | layer4, line1426, Y rounding |
| A45, AAUTO | strict PASS | 5425 | none |
| A50, A125 | strict PASS | 5353 | none |
| R1 | strict PASS | 5835 | none |
| L1 | strict PASS | 5887 | none |

All20 cases admitted in the public validated API, actual CLI exit0;
API/CLI raw G-code SHA256 matched per case, layer markers and total line
counts matched Native. The four Support differences occur in sparse infill,
with the same first difference in official and seeded. Complete ordered
comparator results and all raw streams remain runtime artifacts outside Git.

Input audit compared every regenerated project with its original project,
and official against seeded. All nine uncompressed ZIP members per project
were byte-identical: model, config, metadata and other member data unchanged.
Raw project SHA changed solely because local/central ZIP DOS timestamps changed;
the byte-offset audit verified every differing archive byte lies in those
timestamp fields. The authored STL and unchanged generator hashes match the
manifest. This explains input SHA changes without replacing any input or
rerunning to hide a difference. Oracle wrapper SHA256
1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3.

No B-S production changes belong to this RED.
