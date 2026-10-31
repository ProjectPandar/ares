# Arachne closed exterior inset identity

Status: bounded RED qualification only on the approved combined geometry/walls
baseline. No production change or parity closure is claimed.

## Upstream rewrite boundary

At OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`PerimeterGenerator.cpp:565-566::traverse_extrusions` assigns each closed
ExtrusionLoop the source ExtrusionLine inset index before collection insertion.
`GCode/SeamPlacer.cpp:487-497::extract_perimeter_polygons` accepts a closed loop
with inset zero even when none of its paths has ExternalPerimeter role.

Rust destination: `project_slice/perimeters/arachne/traverse.rs` still constructs
OrderedExtrusionLoop with inset_idx=-1. The existing source-ported consumer in
`project_slice/seam_candidates.rs::extract_collection` already recognizes zero
and scans any ExternalPerimeter sub-path; the missing boundary is closed Arachne
producer metadata, not another role-comparator change. The old broad audit's
first-subpath-only assertion was corrected previously and is not evidence here.

Include only the closed-loop source index transfer. Open MultiPath transfer at
PG595, per-collection fallback, SeamContour physical-boundary metadata, paint,
normal placement, alignment restart, scoring and geometry are adjacent/deferred
slices.
Do not rewrite fallback or synthesize an ExternalPerimeter path/role to hide the
metadata gap. Existing Ares extrusion records are a temporary compatibility shell
around upstream extrusion entities, not a new pipeline design.

## Public witness and failure conditions

Reuse the qualified ordinary stepped cap geometry: 4x4 foot and 12x10 cap beginning
at physical Z0.8, three walls, Nearest mode, no fill/support/paint/Precise Seam.
Change only wall_generator from Classic to Arachne. At Z1 the public Native output
must actually show three fully-overhanging nested wall rings and no Inner/Outer
role. Native replay must be stable. This observes exterior identity through output,
not private metadata counts. The preceding ordinary foot layers remain a useful
control, not an additional test input.

Native-only input 1 is stable on double replay and exposes a 504-byte Z1 object
block with three nested rectangular rings, all Overhang wall, with no Inner/Outer
role; the largest ring lies outside the lower footprint. Input SHA-256:
`31909cf497033d59292683c0224ccc6c6806a368f766a1bac04747e8c64ea64e`.
The existing Classic cap extraction helper is reused verbatim in the same option
group; its original test changes only import/local-helper ownership, not assertions
or input. No Native G-code is committed.

Then run the public validated byte-input API on the same frozen bytes and compare
the complete cap object block strictly with runtime official beta. API/import
failure, missing fully-overhanging exterior, unstable Native output, unchanged
seam behavior or a different geometry/width-clipping residual is not an isolated
production RED for this metadata transfer. Preserve such failed qualifications
and stop or use only the second authorized input; never compensate numerically.

At most two distinct inputs. Geometry C2 and shared small-chain dispatch are now
in the approved source base, but their presence does not itself prove isolation.
A positive metadata witness still requires successful API execution and a visible
exterior/seam-identity difference. No private planner tests, copied expectations,
new diagnostic APIs, sorted/rotated contours, tolerances, forced corners or fixture
rules. Native artifacts stay external. A public RED/RR precedes any source port.
