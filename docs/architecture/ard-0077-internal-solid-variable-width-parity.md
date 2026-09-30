# ARD-0077: Internal-solid variable-width parity investigation

The initial candidate upstream boundary was `VariableWidth.cpp::variable_width` and
`thick_polyline_to_extrusion_paths_2`, called by
`FillConcentricInternal.cpp::fill_surface_extrusion`. Rust destinations are
`project_slice/perimeters/classic/gap_extrusion/variable_width.rs` and its
existing concentric fill adapter. The adapter is a temporary integration shell,
not an independently designed slicing pipeline.

Preserve integer coordinates, f64 thick-line widths and the upstream f32 flow
conversion boundary. Complete ordered G-code remains the acceptance seam;
debugger captures are causal diagnostics, never a replacement oracle or permanent
source-layout assertion. No new tolerance, output filter or fixture branch.

Before implementation, locate the first differing producer boundary. Include
only proven variable-width conversion behavior. If Arachne input or junctions
already differ, revise the spec to cite the actual owning Arachne function and
Rust destination before changing that code. Defer unrelated M73 insertion,
seam occlusion precision and broader Arachne rewriting.

The current KSR RED is layer49/Z9.8:16 unmatched internal-solid moves, first
width .37303 versus .37301. Mesh affine rounding does not change its complete
output. A width symptom alone does not prove the converter is at fault.

Current diagnostic narrows the investigation:44 input contours agree, while
two of86 thick polylines already differ before conversion. No converter repair
is justified by this evidence. Next source boundary to inspect is
`Arachne/WallToolPaths.cpp::generate`, including `removeSmallLines` and
`ExtrusionLine::simplify`; Rust destinations are existing
`arachne/wall_toolpaths.rs::generate` and its postprocessing modules. These are
candidate investigation boundaries, not authorization to port their unrelated
behavior. Confirm the first differing junction stage before revising the
implementation spec to a proven owner.

Prepared-outline comparison now matches all21 target vertices in order.
Native/Rust toolpaths differ before stitching, so the current investigation
owner is `SkeletalTrapezoidation.cpp` construction and `generateToolpaths`,
mapped to `arachne/trapezoidation.rs::{new,generate_toolpaths}` and its existing
builder/filtering/transition modules. Neither the converter nor outline
preparation is a demonstrated repair target. Include only the eventual proven
junction-generation mismatch; keep broader Arachne behavior deferred.

Actual constructor parameters and all13 beading-factory inputs now match for
the target domain. The107 active initial skeleton nodes also match in point
coordinates and boundary distance. Rust's138 storage slots include31 deleted
nodes and must not be confused with107 active native list nodes. Establish
half-edge adjacency and stage state before treating construction as a repair
target; matching nodes alone does not prove topology or traversal equivalence.

Coordinate-connectivity multisets for212 active half-edges now also agree.
Initial, central-classified, central-filtered and bead-count-updated snapshots
match node distance/bead state and edge type/central flags. This excludes a
demonstrated mismatch in those captured states, not traversal identity/order.
Next inspect noncentral filtering and transitioning ribs; no earlier producer
repair is supported. Correct optimized debug locations before interpreting
stage names. Complete ordered G-code is still RED, including native reference
run-to-run M73 placement differences; diagnostics never authorize masking them.

The first captured stage-state divergence is now within
`SkeletalTrapezoidation.cpp::generateTransitioningRibs`: after noncentral
filtering the recorded107-node/212-edge state matches; after transitioning
ribs native has110 nodes/218 edges and Rust113/224. Rust adds the later
divergent endpoint(-12964813,-10241430). Narrow the candidate repair boundary
to that function's middle/end generation, filtering and applyTransitions,
mapped to existing `arachne/trapezoidation/transitions` modules. Identify the
first differing substage and public minimized reproduction before repair;
stage association alone does not prove which predicate or arithmetic is wrong.

Verified owner is `SkeletalTrapezoidation.cpp::applyTransitions`: retain the
original node identities for snap comparisons throughout one edge's endpoint
batch. Rust destination is
`trapezoidation/transitions/apply.rs::apply_transitions_on_edge`, using its
already captured from/to IDs instead of reading mutated edge endpoints.
Leave transition calculation, traversal, graph insertion and conversion
unchanged. Supersede the private test requiring the mutated-endpoint behavior
with a public complete-output regression. Fixture polygon coordinates are
mesh inputs, not expected private junctions or source-layout assertions.
