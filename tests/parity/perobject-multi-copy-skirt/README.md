# Per-object multi-copy skirt Native-active RED inputs

Input-only 3MFs and SHA256 manifest. Only skirt_type differs: target perobject,
control combined. Two owners/three placements: object2 at X80/X140,Y110,Z2;
object3 at X110,Y110,Z2; Z scale0.4. ByLayer, ordinary combine_brims0, outer_only
brim8/gap0, skirt2/height1, one filament, no support/raft/tower.

Source: Print.cpp:2794-2920; Brim.cpp:36-45,573-581,914-977;
GCode.cpp:4486-4531. Approved proposal:
docs/architecture/per-object-multi-instance-brim-guard-proposal.md.

Native is generated FIRST at runtime. Native-active assertions require two brim
owner blocks, one skirt block, and positive outer-wall extrusion at ALL three
model placements; footer settings alone are insufficient. Model-position ranges
are activation anchors ONLY, never comparison tolerances or acceptance substitutes.
The tests then call validated API and real CLI, asserting complete strict ordered
Native equality (unchanged generator-only comparator) and raw API==CLI for BOTH
cases. Before GREEN, target must truly fail its unsupported result and control
must pass; divergence-as-success assertions are prohibited. No Native outputs.

GREEN may retire only the existing ByLayer instance-count guard applicability.
ByObject restriction remains, with an accurate comment. No geometry/emitter,
other guard, parser, estimator, comparator or normalization changes.
