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

Fresh preflight after rebase onto main73e8e712e: Native official and seeded
17/17 exit0 each, with bottom1 vs4 local lower-layer solid/sparse differences:
/tmp/worker3-modifier-bottom-native-e_ef9axn/evidence.json and native-local-effect.json.
Earlier preflight root /tmp/worker3-modifier-bottom-native-4a9dtmt3 is retained.
All archived files are authored inputs, with no Native products.
