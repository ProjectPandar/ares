# Inner-only brim inputs

Approved SPEC `696cd308a` ports `libslic3r/Brim.cpp::outer_inner_brim_area`
for one object, instance, model part, group and filament. These input-only
3MF archives contain no Native outputs or goldens. `generate.py` rebuilds
all nine archives from the recorded legacy O0 input and fixed settings.
Run `python3 tests/parity/inner-only-brim/generate.py` at the repository root.
`provenance.json` records hashes, polygons, options, mesh counts and volume.
The generator verifies one reverse edge for every directed triangle edge.

Each solid is the extrusion of a 30x30 mm square minus the listed holes,
placed at (110,110), z=0..1. Top/bottom shells and sparse infill are disabled;
two classic walls preserve the real first-layer hole. EFC, skirt, support,
prime tower and multi-material are disabled. The rectangular hole is 12x12;
concave, two-hole and narrow-neck cases use the manifest polygons.

Run the `orca_parity` tests filtered by `test(inner_only_)` in both reference
modes with control-set-v2, jobs2/threads2 and fresh cache/artifact roots.
Reuse `test(legacy_brim_)` for the existing six unchanged outer/ear controls.
The suite saves Native effective footer settings, full Native/API/CLI streams,
commands, producer identity, first byte differences and positive XY/E brim
motion witnesses. Endpoint/midpoint-in-hole checks supplement the complete
ordered-byte comparator; they do not replace it. Native or public admission
errors are not geometry RED. Zero/sub-pair width and no-hole are inactive
controls; the other six cases must have actual Native brim extrusion.
