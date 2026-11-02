# Internal bridge output and fill projection

## Status and rewrite boundary

Investigation and independent public RED, not a production port or GREEN claim.
The source is OrcaSlicer beta revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

The owner is `PrintObject::bridge_over_infill`, not a newly designed slicing
pipeline. Its candidate expansion, direction selection and surface rewrite feed
`Layer::make_fills` / `group_fills` in `Fill/Fill.cpp`. The Rust destinations are
`project_slice/prepare_infill/bridge_over_infill` and
`project_slice/group_fills/params/projection.rs`. Existing Rust helpers are a
partial compatibility shell around those upstream operations, not an alternate
behavioral specification.

| Upstream boundary | Rust boundary | Contract to investigate |
| --- | --- | --- |
| `PrintObject.cpp:3872–3886` candidate components | `candidate_bridge_area.rs` | Intersect expanded candidates as ExPolygons; retain or discard a whole component with its holes based on unsupported-area overlap. |
| `PrintObject.cpp:3480–3552,3913–3959` direction | `automatic_bridge_angle.rs`, `candidate_bridge_angle.rs` | Real anchors, geometric fallback, source voting and custom override; the configured turning-pattern gate is only for Hilbert/Octagram. |
| `PrintObject.cpp:3961–4043` reconstruction and clipping | `candidate_anchored_bridge.rs`, `candidate_collision_reconstruction.rs`, `candidate_bridge_postprocessing.rs` | Preserve polygon topology, selected angle and source operation order through reconstruction, smoothing and clipping. |
| `PrintObject.cpp:4073–4107` surface rewrite | `internal_bridge_surfaces.rs`, `transaction/surface_rewrite.rs` | Copy the original solid surface metadata, retag as internal bridge, assign candidate angle, union candidate polygons as ExPolygons. |
| `Fill/Fill.cpp:940–1080,1507–1513` group/runtime parameter projection | `group_fills/params/projection.rs` | Project effective pattern, role, density, multiline, bridge angle and configured bridging flow from the rewritten surface. |
| `Fill/FillBase.cpp:411–418` bridge direction | `fill/rectilinear/surface.rs` | A nonnegative surface bridge angle overrides ordinary layer alternation before traversal. |

## Frozen public inputs

The inputs are complete project byte streams, not reconstructed option lists.
The fixtures live under `tests/parity/internal-bridge-output/`. Historical saved
results came from the cubic breadth run; independent direct-target RED now
executes all three fixtures on the bridge-prerequisite parent.

| Fixture | Sparse pattern / count | SHA256 |
| --- | --- | --- |
| `honeycomb-count2.3mf` | honeycomb / 2 | `e43bf5ac79bcb39eae1cfeb0f2ab0d93b97627fb83368473bb1760f49e16a07c` |
| `triangles-count1.3mf` | triangles / 1 | `58efc7d45f6ae6df74c711a7a7d5caa80f2cbf8e01ab0045039d0b3b9c0f8049` |
| `triangles-count2.3mf` | triangles / 2 | `29cb5b6f383537a1c5da393bba66b3f5e61b83b01df3b1e52cdde2561bcc0c5e` |

Both configure rectilinear top and internal-solid patterns, 30% sparse density,
100% internal-bridge density, thick internal bridges, no model alignment,
empty rotation templates and no internal bridge angle override. Neither sparse
pattern enters the Hilbert/Octagram direction gate. Upstream `Fill.cpp:978–981`
projects these bridge surfaces to Rectilinear at full density;
`Fill.cpp:1017` limits configured multiline to sparse internal infill, so the
bridge count is one even for the honeycomb count-two project.

A historical triangles count-two test first requires equality for the count-one
baseline and fails there. Its saved input is identical to the second fixture.
It is not independent count-two coverage and must not serve as target evidence.
The new count-two fixture changes only the `fill_multiline` project-setting value
from `"1"` to `"2"`; every other decompressed ZIP entry and configuration field is
unchanged. Its direct test has no count-one baseline assertion.

## Established facts and unknowns

The saved public outputs each contain an Internal Bridge block at print Z 3.4.
Their serialized width and height comments both read `0.389872`. This establishes
only emitted comment equality, not exact Native flow fields. Honeycomb already
has different ordered bridge entry/travel records; triangles also has visibly
different extrusion segment directions. Neither observation establishes Native
surface geometry, holes, internal angle, spacing or traversal-entry equality.
Counting raw positive-E XY moves while the Internal Bridge role is active gives
410 Native versus 408 Rust records for honeycomb and 410 versus 381 for triangles.
These are saved-output diagnostics, not an acceptance comparator or proof of a
shared defect; retain the original complete ordered streams.

There is a concrete source mismatch before traversal: upstream filters whole
ExPolygon components, while Rust `candidate_bridge_area.rs` filters individual
rings returned by a flat polygon intersection. A hole can be discarded separately
from its contour or retained independently of its parent. This is a candidate
port boundary, not yet the demonstrated cause of either frozen output failure.
The two inputs are not assumed to share a root cause. No Native intermediate
observation is available for these inputs.

## Included work and validation gate

1. Add one independent full-output E2E test for each frozen project. Each test
   directly evaluates its target, checks its input SHA, runs Native twice, then
   uses the unchanged public API/CLI complete ordered-byte comparator. Preserve
   feed, M73, metadata and footer records; do not introduce a block-only verdict.
2. Record actual RED on the bridge-prerequisite parent, rather than relying on
   results from an older fill branch. Retain repeatable input/output artifacts
   and verify API/CLI identity independently if the full comparison panics first.
3. Before any production port, localize ordered surface contours/holes and
   metadata at candidate selection, bridge rewrite and fill projection. Distinguish
   preexisting candidate topology from a direction-voting or projection defect.
   A topology repair requires evidence that the component/ring distinction is
   actually exercised, not just a plausible static mismatch. If that distinction
   is inactive, return a localization RR rather than porting it as a speculative
   fix for these inputs.
4. Request bounded authorization before any Native intermediate observation.
   Final G-code is not a substitute for that evidence. Temporary Rust probes,
   if used for follow-up localization, must be removed and byte restoration
   verified; they are not product interfaces.

The public seam is complete project bytes through core API and CLI to complete
G-code, with Native as independent expected behavior. This is not an isolated
helper test or an assertion against a private probe.

## Failure modes to distinguish

- A disconnected component without unsupported overlap must be rejected together
  with its holes; a retained component must keep all holes.
- Hole rings independently tested as filled polygons can falsely influence angle
  samples and bridge reconstruction before any rectilinear graph is built.
- Correct candidate topology does not imply equal anchors, selected angle,
  collision direction reuse, clipping order or rewritten metadata.
- Matching emitted width/height comments can conceal unequal spacing or flow.
- A stale baseline, duplicated input or baseline-first assertion can prevent the
  intended target from executing.
- A public complete-output failure may remain due to configuration/footer
  ownership even after geometry improves; report it as FAIL, not inferred GREEN.

## Deferred behavior and non-negotiable decisions

No rectilinear/monotonic graph, path reversal, rounding heuristic, writer,
footer/configuration, smoothing policy or unrelated sparse-fill repair is
included. Turning-pattern scan refinement and contact restoration remain separate
upstream slices; these two configured patterns do not activate those branches.
The new triangles count-two target is directly executed; its result must remain
separate from the historical baseline-only failure.

Do not mutate the approved prerequisite branch or stack unapproved footer work.
Any later production change needs a source-cited, localized scope and separate
port commit, same-inventory breadth with zero PASS-to-FAIL transitions, rustfmt
and strict all-target Clippy. The limited motion-block GREEN exceptions granted
to other fill slices do not apply here. Core changes, if later justified, must
remain rendering-neutral and portable to browser WASM, Windows, macOS and Linux.
