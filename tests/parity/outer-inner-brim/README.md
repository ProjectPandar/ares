# OuterAndInner brim inputs

Approved SPEC53b22ddc9 ports Brim.cpp:448-570,661-711,813-875. These ten
3MF archives are input-only meshes/configuration; no Native output is embedded.
Run `python3 tests/parity/outer-inner-brim/generate.py` from the repository root.
ZIP metadata is deterministic. The generator verifies every directed triangle
edge has exactly one reverse and signed volume is positive; provenance records
input hashes, contours, holes, counts and requested width/gap.

All cases use one object/instance/model-part volume/group/tool, ordinary
first-layer slices, two classic walls, zero infill/top/bottom, and disabled
support, prime tower, skirt and EFC. The two-islands input has disconnected
footprints within one mesh/volume, with one hole per island.

Tests run original Native, the validated byte API and real CLI, save complete
G-code/settings/commands/comparisons under ARES_PARITY_ARTIFACT_ROOT, and add
positive first-layer Brim XY/E witnesses inside holes and outside contours.
The unchanged generator-only full comparator remains the acceptance check.
Inactive zero/sub-pair widths emit neither region; no-hole emits outer only.
Native/API/CLI admission must succeed before a mismatch qualifies as RED.

Use canonical control-set-v2 jobs2/threads2 with a fresh Native cache/artifact
root each mode/run. Existing inner-only and N/O/E controls stay unchanged.
