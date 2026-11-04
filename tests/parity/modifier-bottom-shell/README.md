# ParameterModifier bottom-shell input matrix

SPEC 7f0717e07; source PrintApply.cpp:1032-1088 and
PrintObject.cpp:3664-3710/2252-2280/3973-3995.
Run `python3 tests/parity/modifier-bottom-shell/generate.py` from any directory.
The generator records source hashes and parameters in provenance.json.

The unchanged base prism is 10 mm tall, with a right-half modifier reaching
the bottom. Parent bottom count=2, thickness=0; modifier bottom=0/1/4/equal2,
no override, disjoint control, or overlapping later bottom1. Classic horizontal
variants use ensure_vertical_shell_thickness=none; others use ensure_all.
Original top settings and all top fixtures remain unchanged.

Native official and seeded slice all 17 cases with exit 0, with bottom1 vs 4
local lower-layer solid/sparse differences.
All archived files are authored inputs, with no Native products.
