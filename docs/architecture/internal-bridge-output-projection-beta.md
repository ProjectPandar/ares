# Internal bridge output and fill projection

## Status and rewrite boundary

Component-filter port implemented after independent public RED and measured
scope approval. Strict complete-output acceptance still awaits the separately
owned preset/footer prerequisite; no full-output GREEN is claimed.
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

## Measured trigger and approved port scope

Read-only Native debugger captures and temporary Rust diagnostics now establish
that the component/ring distinction is exercised in both the honeycomb count-two
and triangles count-one projects. The measurements precede any production port.
All four Native observation outputs match their public RED streams except the
permitted generator timestamp. Optimized-out variables at the earlier breakpoint
are excluded; the subsequent direction breakpoint exposes all compared fields.

At layer 16, print Z 3.4, the following are ordered, coordinate-exact matches:
original candidate polygons, expanded polygons, deep infill area, unsupported
area, and real anchors. The anchor counts are 78 for honeycomb and 60 for
triangles. Native's retained area is exactly the two-ring Rust intersection
before filtering. Native retains both contour and hole; Rust's independent-ring
predicate removes the hole and keeps only the contour. This is the first measured
geometric mismatch at the candidate boundary, not a traversal inference.

A shadow diagnostic invokes the existing Rust automatic-angle helper with the
unfiltered two-ring intersection and the same anchors, without using its result
in the pipeline. Its f64 result is exactly equal to Native for both projects:

| Project | Native f64 angle | Actual Rust f64 angle | Two-ring shadow f64 angle |
| --- | --- | --- | --- |
| honeycomb count-two | `3.9269908456133424` | `3.92699087173241` | `3.9269908456133424` |
| triangles count-one | `2.356194536926903` | `3.403392055427836` | `2.356194536926903` |

At the Rectilinear entry, both implementations have the same ordered contour and
hole, thickness -1, thickness-layers 1, and spacing `0.43987178802490234`. The
selected bridge angle has already diverged. The honeycomb f32 angles are
`3.9269907474517822` and `3.9269909858703613`; triangles uses
`2.356194496154785` and `3.4033920764923096`. No modulo-angle or tolerance
acceptance is used. Exact full Native Flow fields were not captured; matching
width/height comments are not promoted to flow equality.

Limiting boundary polylines also differ: Native has four, Rust three. That
consequence is retained in the evidence, not ignored. These automatic-direction
calls use the nonempty, ordered-equal real anchors, not the fallback boundaries.
The restored hole in the final surface therefore does not establish that the
hole was present during direction voting.

All three Rust diagnostic outputs, including the unused shadow evaluation, are
byte-identical to the uninstrumented RED outputs. All temporary probes are
removed; rebuilding the restored code reproduces the same complete bytes for
all three fixtures. No diagnostic interface or production fallback is retained.

The approved production slice is only `PrintObject::bridge_over_infill`,
`PrintObject.cpp:3872–3886`: intersect expanded candidates into ExPolygon
components; test unsupported overlap for the whole component including holes;
retain or discard that whole component; flatten the retained components in
source order before the existing limiting-area union. The Rust destination is
`candidate_bridge_area.rs`, using the existing ExPolygon boolean operations.
This replaces the flat-ring predicate; it is not an additional pattern-specific
path. Keep offset arithmetic, anchors, automatic angle voting, overrides, bridge
flow, reconstruction, final surface rewrite, fill projection and traversal
unchanged. The approved slice also includes the source subject-bounding-box
clipping exactly at both intersections: the expanded-candidate bounds for deep
infill, and each component's bounds for unsupported area, each inflated by
`SCALED_EPSILON`. Reuse the existing bbox clipper and scale the source epsilon
with `CoordinateScale`. The existing input struct carries that scale through a
mechanical caller change; no caller geometry or direction behavior changes.

The investigation and direct RED commits are replayed onto
`44fb3e6912efec80b2a12e01df6a44045daf3651` before any GREEN implementation.
Fresh requalification there still yields three complete-output FAILs, with all
Native double replays equal and all API/CLI pairs byte-identical. Complete Ares
bytes for each input are unchanged from the original RED. The count-two triangles
fixture still differs only at the generator and the two known footer keys, so
the preset/footer prerequisite is not satisfied for these frozen inputs on this
base. Do not repair or mask that prerequisite inside the component port, and do
not label the port GREEN while the complete-output gate remains unsatisfied.
Require all three complete-output targets to pass after an approved port and its
separately owned prerequisite. The existing count-two triangles fixture is a
control with raw motion and M73 already exact on the original RED base, not a
third demonstrated geometric failure. A future port must also run identical
before/after bridge/fill breadth inventories with zero PASS-to-FAIL transitions,
rustfmt and strict all-target Clippy. Measurements alone did not change
production behavior; implementation is restricted to the approved component
filter and bbox clipping unit above.

## Post-port requalification

Fresh read-only Native observations and temporary Rust observations agree
exactly on the ordered candidate area, anchors, boundary polylines, and
Rectilinear entry ExPolygon for both failing geometric cases. Candidate angles
are exactly `3.9269908456133424` for honeycomb count two and
`2.356194536926903` for triangles count one, including equal projected f32
angles. Both now have four boundary polylines with identical point order; no
angle or boundary difference remains in these observations. Full Native Flow
fields were not measured, so no full Flow equality is inferred.

Temporary observations were removed and rebuilt CLI outputs for all three
fixtures are byte-identical to their uninstrumented post-port outputs. The
triangles count-two control is also byte-identical before and after the port.
The exact 235-test breadth inventory remains 67 PASS / 168 FAIL, with zero
PASS-to-FAIL or FAIL-to-PASS transitions. Rustfmt and strict all-target Clippy
pass without new suppressions.

All three complete-output tests remain enabled and FAIL. Raw motion and M73
are exact for all three; exhaustive raw differences contain only generator
metadata and the two separately owned footer keys. The unchanged complete
comparator's first differences are `compatible_process_expression_group` at
line 18732 for honeycomb count two, line 6987 for triangles count one, and line
10557 for triangles count two. `pellet_flow_coefficient` is the other footer
residual. These diagnostics are not a complete-output GREEN exception. Full
acceptance requires the preset-refresh prerequisite and a fresh strict run.
