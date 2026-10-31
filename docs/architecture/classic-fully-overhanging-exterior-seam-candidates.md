# Classic fully-overhanging exterior seam candidates

## Source boundary

OrcaSlicer v2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/GCode/SeamPlacer.cpp:487-501::extract_perimeter_polygons`,
recognizes a closed exterior loop when any path is ExternalPerimeter **or** its
`inset_idx` is zero. The latter preserves exterior identity when every path is
OverhangPerimeter. This is an upstream extraction rule, not new Ares seam logic.

Rust destination:
`crates/ares-core/src/project_slice/seam_candidates.rs::extract_collection`.
Its current closed-loop arm already scans every path for ExternalPerimeter; that
behavior is included but requires no rewrite. Only the inset-zero supplement is
missing. Classic `OrderedExtrusionLoop::inset_idx` already preserves loop depth
in `project_slice/perimeters/classic/entity_collections/types.rs`.

## Included and deferred behavior

Include classic closed-loop exterior recognition with existing path collection,
candidate order, flow widths and geometry unchanged. Preserve open Path/MultiPath
classification and the existing any-External-path arm. The existing Rust extractor
is a partial compatibility shell around the cited upstream extraction boundary.

Defer per-collection fallback changes, contour/hole metadata, alignment/string
changes, loop-normal projection, paint, Precise Seam, Random and emission changes.
Arachne inset propagation (`PerimeterGenerator.cpp:565-566,595`) is another source
boundary and is not silently included in this classic witness. No new clipping,
model-import API, geometry compensation or seam-placement architecture is required.

## Public qualification and RED contract

Use an ordinary classic stepped solid, Nearest seam, no painting or Precise Seam,
no supports, shell or sparse fill. At the first expanded cross-section, the lower
footprint is strictly inside the exterior wall. Multiple walls make the old
whole-collection fallback distinguishable from the retained exterior polygon.
This input geometry is a proposed witness, not proof that output must differ.

Qualify at most two distinct inputs against the official pinned beta producer.
Native double replay must be stable, and emitted moves must establish a complete
exterior ring containing only Overhang wall extrusion, without an Outer wall path
on that ring. Nearest avoids the unrelated beta string-fitting changes.

After Native qualification, use `ares_core::slice_project_plate_validated` on the
same frozen bytes. A public RED must demonstrate a strict runtime Native/API
difference in the affected object-layer block. Freeze that RED before production
changes. Runtime Native bytes supply expectations; no committed Native output,
private planner assertion, diagnostic API, synthetic paint flags or computed
algorithm expectation is admissible. Keep the parity root registration unchanged.

## Failure modes and stop conditions

- Native rejects the input or replay changes: qualification failure, not RED.
- The outer ring retains an Outer wall path or is not fully overhanging: wrong witness.
- Public API rejects the input or produces the same target output: no qualified
  behavior RED for this slice; do not invent an internal test or patch production
  to search for sensitivity.
- A mismatch belongs to clipping, path materialization, travel or another source
  boundary: do not absorb it with a tolerance, tie key or fixture-specific rule.
- Both permitted inputs fail to provide a witness: stop this candidate and return
  to the separately source-cited immediate-lower-layer string-restart boundary.

Any GREEN must leave the approved RED bytes/comparison unchanged and add only the
missing upstream predicate. Directed public evidence and fmt/Clippy qualify that
change; broader beta qualification and ledger linkage remain integration concerns.
