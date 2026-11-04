# Spec: centered sparse-anchoring fill reference

Upstream OrcaSlicer2.4.2 commit8500fcdccaa10b5099ac20d252af3a7c560046f1:
`Fill/Fill.cpp:1394-1442 Layer::generate_sparse_infill_polylines_for_anchoring`
reads `this->object()->bounding_box()` and sets the fill object's bounding box.
`Print.hpp:347 PrintObject::bounding_box` constructs symmetric integer bounds;
`FillBase.cpp:286-290 Fill::_infill_direction` therefore uses center(0,0).

Rust destination is `project_slice/prepare_infill/bridge_over_infill/
sparse_anchoring/mod.rs::generate_sparse_infill_polylines_for_anchoring`.
Replace its slice-contour-center approximation with the upstream zero reference.
The rest of this module remains a compatibility shell around the cited upstream
method, not an independently designed Ares pipeline.

Included: anchor reference selection and deletion of the now-unused
`fill_entities/flow.rs::object_center` plus its re-export. Deferred: the separate
contour-derived extended bbox approximation, bridge detection/angle/flow math,
PerimeterGenerator extra overhangs and unrelated Qidi causes. No new guard,
normalization, numerical tolerance, platform I/O or public API.

TDD seam: unchanged public saved3MF/options → complete G-code replay against
original Orca2.4.2, generator-only normalization. On main94a0a629, replay all16
residual inputs before production edits. After edits repeat16, full68, previously
restored52 and823 stock controls, plus8 fresh official references retained from
the prior slice. Preserve every non-PASS; claim only observed recovery and do not
waive assertions. Workspace Nextest/fmt/default Clippy/actual core+adapter WASM
build are mandatory. Existing lint debt is not fixed or suppressed here.
