# ByObject/per-object skirt public inputs

Input-only 3MFs from existing per-object-skirt/disjoint fixture. No Native outputs.
Source Print.cpp:2548-2557,2780-2977; GCode.cpp:2730,3280-3374,4465-4483,5334.
Two distinct separated objects, one filament, no brim/support/raft/tower.
Target ByObject/perobject; controls ByLayer/perobject and ByObject/combined.
Only sequence/skirt config differs; SHA256 recorded in manifest.
combine_brims=0 is a verified current pinned-source key, no brim behavior claimed.

Runtime Native first; target binds two skirt blocks to sequential object output,
second after origin transition and before second object extrusion. Every case
asserts successful API/CLI and strict whole Native equality plus API==CLI.
Target must fail on current UnsupportedProjectFeature until approved GREEN.
Shared-group Native slicing-error reachability is not claimed. A separate
skirt_distance35 discovery input rejected earlier at sequential clearance and
is not committed as a fabricated shared-group witness. Shared groups must remain
guarded in any later production change; no geometry/clearance bypass permitted.
