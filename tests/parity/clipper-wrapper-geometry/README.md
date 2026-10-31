# Clipper wrapper geometry projects

Input-only projects for OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`. No Native output, G-code fragment
or golden is committed. Native streams are generated and replayed at runtime;
the existing generator-line-only comparator checks complete ordered API/CLI
output, including the footer. Raw API and CLI output must also match each other.

Generate from the repository root with:

```sh
python3 tests/parity/clipper-wrapper-geometry/generate.py
```

The source archive supplies packaging and explicit printer settings, not a beta
reference output. `provenance.json` records its hash, every frozen input hash,
height bands, XY regions and settings. Each generated mesh is checked for positive
signed volume and exactly one reverse for every directed triangle edge. ZIP
metadata is fixed. These mesh checks validate inputs, not expected slice output.

| Project | Behavior exercised |
|---|---|
| `nested-hole-island` | Two nested ring components: outer contour, hole, island and its hole. |
| `holes-touch-after-compensation` | Watertight separated input holes; compensation 0.4 brings their near corners together. |
| `collinear-boundary-with-thin-spike` | Grid-aligned collinear runs and a 0.0001-wide rectangular spike, with simplification/closing disabled. |
| `round-offset-short-edge-below-threshold` | Round ear-brim offset, gap 2, spike width 0.0098. |
| `round-offset-short-edge-at-threshold` | Same offset, explicitly gridded 0.01-wide spike. |
| `round-offset-short-edge-above-threshold` | Same offset, spike width 0.0102. |
| `coincident-width-overhang-intersections` | Arachne step/ledge with 0.5 nozzle and line widths; shared supported corner plus newly clipped overhang crossings. |

The offset trio brackets the wrapper's `abs(delta*0.005)` filtering policy,
with a scaled gap of 2000000 and nominal threshold 10000. The at-case tip endpoints
use Y 0.015625 and 0.025625002; source f32 input conversion followed by scaled
integer conversion maps them to 15625 and 25625. A four-unit spike and disabled
ear decimation retain the input feature; ear angle 180 admits convex corners.
Round subdivisions and cap/arc policy are observed through complete emitted brim
paths, not normalized geometry. Whole-stream equality does not prove every
intermediate filter decision. The width-bearing case uses binary-exact
half-unit outline steps and quarter-unit nominal center offsets to align a shared
corner. It also requires Native overhang-wall activation. Whole-stream parity is
not an internal callback-count or exact intermediate-coordinate coverage claim.

Owners at the pinned source: `ClipperUtils.cpp:append_offset_path`,
`c2_offset_setup`, `c2_clip`, `c2_to_expolygons`, `c2_attach_stray_holes`;
`ClipperZUtils.cpp:48–80:clip_zpaths`;
`PerimeterGenerator.cpp:321–390:clip_extrusion`,
`:1488–1490,:2668–2670` lower-slice nozzle-radius offset;
`deps_src/clipper2/Clipper2Lib` engine/offset implementations.
TriangleMeshSlicer canonical ordering/coplanarity remains a separate rewrite slice.

Registration is in the existing `runner/native_stream_cases.rs` group, not the
`orca_parity.rs` root. Select the approved beta producer with `ARES_PARITY_ENV`
and run only the `clipper_wrapper_geometry::` parity filter. Missing Native is a
failure, not skipped parity. Runtime artifacts include both Native captures,
API/CLI streams and comparison metadata; they are external to the repository.
