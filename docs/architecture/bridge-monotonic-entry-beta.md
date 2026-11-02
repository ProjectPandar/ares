# Beta bridge-over-infill output and monotonic entry

## Rewrite boundary and dependency

Source revision is Orca beta `345efec64c7b95a05ce5cf96df0dde66a158835c`.
Begin with `PrintObject::bridge_over_infill`
(`src/libslic3r/PrintObject.cpp:3088–4274`), particularly the committed
`stInternalBridge` surfaces at `4073–4086` and final region collection at
`4106–4107`. The source copies the original solid surface metadata, replaces
its type and bridge angle, and emits the ordered `union_ex(cs.new_polys)`
components. Output geometry, holes, metadata, angle and order must be localized
before treating a later fill traversal as the owner of a public motion mismatch.

The Rust destination is
`project_slice/prepare_infill/bridge_over_infill/transaction.rs`,
`transaction/surface_rewrite.rs`, `internal_bridge_surfaces.rs`, and the reached
candidate/commit helpers. Existing Ares staged records are compatibility shells
around the upstream surface transactions, not a separate pipeline design.
This branch stacks on the approved bridge-limiting-offset tip `a3182a424`,
which owns the same `PrintObject` transaction and its corrected boundary units.
Do not modify the approved parent branch or the independent multiline branch.

If output and fill-entry inputs are proven equal, continue through upstream
`Fill.cpp:978–981` (Monotonic/MonotonicLine top pattern maps bridge to Monotonic),
`FillRectilinear.cpp:3504–3512` (`FillMonotonic::fill_surface` sets monotonic),
and `fill_surface_by_lines` at `2753+`. Rust destinations are
`project_slice/group_fills/params/projection.rs`,
`project_slice/fill_entities/monotonic.rs::expolygon_polylines`, and
`fill/rectilinear/surface.rs::fill_monotonic_surface`.

## Public evidence and unknowns

Three frozen plane-path-object-bounds inputs first differ in positive XY
extrusion at Internal Bridge Z=9. Native has 52 bridge extrusion records in each;
Rust has 35 for both Hilbert inputs and 44 for the Octagram input. Their sparse
pattern names do not establish `FillPlanePath` as owner of the bridge mismatch.
The actual bridge fill is Monotonic, density 100%, multiline 1. Whole-output
first differences occur earlier and include M73; retain them as failures.

Neither equal serialized width/height nor an apparent common line angle proves
entry flow or surface equality. No Native intermediate snapshots exist for
these three inputs. The older layer-67 observation is a different input and is
not equality evidence here. Ares-side probes and static source comparison may
localize a concrete mismatch, but must not be described as Native observations.
If they cannot establish entry equality, report the evidence gap and request a
bounded observation in an RR before running any Native observer. Current budget
is zero. Standard public parity replay remains the authorized test mechanism.

## Executed angle-selection mismatch before monotonic fill

The Ares-only captures establish a difference before monotonic traversal:

| Input | Candidate and rewrite angle (f64) | Monotonic entry angle (f32) |
| --- | ---: | ---: |
| Hilbert, compensation 0 | 4.7123887041029775 | 4.7123885 |
| Hilbert, compensation 0.1 | 4.712388855299633 | 4.712389 |
| Octagram, compensation 0.1 | 1.7671458676442586 | 1.7671459 |

All three execute candidate layer 44 / region 0, with 5, 5, and 8 sparse
anchors respectively. Lower layer 43 / the same region has one Internal square,
no holes, contained strictly inside the candidate area. Thus the same-region
sparse-overlap condition is true in the observed Ares data. Direction is 45
degrees, sparse rotation template empty, model alignment disabled, model
rotation zero, and internal bridge override zero.

Upstream `PrintObject.cpp:3913–3942` first selects the lower-layer configured
rotation for anchored Hilbert/Octagram with same-region sparse overlap. For
these inputs, `Fill.cpp:84–91` returns `deg2rad(45)` for the empty template;
adding `0.5 * PI` and normalizing gives `2.356194490192345`. The Ares
`candidate_bridge_angle.rs` has no such branch: it invokes geometric detection
unconditionally, and `automatic_bridge_angle.rs::pattern_adjustment` adds a
Hilbert `PI/4` or Octagram `PI/16`. These are concrete source inconsistencies on
the captured path, not a need for a numerical tolerance or a monotonic repair.
Native's ordered bridge motions are diagonal, consistent with the configured
selection, but no exact Native intermediate angle is claimed to be captured.

The Ares bridge-over-infill output and monotonic entry retain the same ordered
square and no holes: half-size 4,223,233 for compensation 0, and 4,323,233 for
compensation 0.1. Metadata is InternalBridge, thickness -1, thickness_layers 1.
Entry flow is width/height 0.38987178, spacing 0.4398718, mm3_per_mm
0.11938051879405975, bridge true, density 100%, multiline 1. This is Ares-side
transfer equality only. Equal Native serialized width/height does not prove
all Native flow fields or polygons equal. Entry is already inconsistent in
angle, so there is no basis to advance into monotonic graph traversal.

The next repair boundary is the configured lower-region angle gate at
`PrintObject.cpp:3913–3942` and its reached `Fill.cpp` rotation calculation,
with the existing custom-angle override still applied afterwards at `3944+`.
Do not hard-code 135 degrees: retain lower-layer ID, same-region sparse-overlap,
real-anchor, template and model-alignment semantics, and the geometry fallback
when that gate is inactive. Sampling resolution, contact restoration, and other
adjacent turning-pattern behavior remain separate until independently required.

All probes were temporary, restored, and emitted only to stderr. Their CLI
outputs are byte-identical to the uninstrumented RED CLI outputs for all three
inputs. No Native debugger or observation was run.

## Included and deferred behavior

Include ordered bridge-over-infill output, angle and flow localization, followed
by the monotonic entry only if its inputs match. A later repair must cite the
specific reached upstream operation and preserve its arithmetic and ordering.
No production change is authorized merely by this investigative boundary.

Defer plane-path generation changes, global path reversal, generic collinear
cleanup, geometry backend replacements, footer workarounds, arc/writer changes,
and any unproven adjacent bridge/filter discrepancy. Geometry primitive changes
require their owning worker's coordination. Native builds and new Native
observations are not authorized by this document.

## Verification gate

1. Commit this source-owned ARD and public-output RED separately before a repair.
2. Test each frozen input directly, avoiding the historical compensated test's
   baseline assertion coupling. Qualify two Native replays and use the unchanged
   public API/CLI complete ordered-output comparator with SHA assertions.
3. Capture Ares bridge surface output and monotonic-entry geometry/metadata,
   angle, spacing and flow using temporary diagnostics only. Preserve evidence
   externally; remove all probes before any production commit.
4. Do not advance to monotonic traversal on presumed entry equality. If Native
   entry equality is unproven, submit an RR identifying the precise missing
   observation rather than guessing a fix.
5. A proven repair needs a separate commit, the three complete-output results,
   same-inventory bridge/fill regression transitions, rustfmt and all-target
   Clippy with warnings denied. Do not weaken feed/M73/footer acceptance or
   ignore failing tests. Inherited failures remain failures unless an explicit
   acceptance ruling states otherwise.
