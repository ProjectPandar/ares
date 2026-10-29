# ParameterModifier top-shell input matrix

SPEC 9b7ef7b71; source PrintApply.cpp:1032-1088 and
PrintObject.cpp:3664-3710/2223-2251/3973-3995.
Run `python3 tests/parity/modifier-top-shell/generate.py` from any directory.
The generator records source hashes and parameters in provenance.json.

The unchanged base prism is 10 mm tall, with a right-half modifier reaching
the top. Parent top count=2, thickness=0; modifier top=0/1/4/equal2, no override,
disjoint control, or overlapping later top1. Classic horizontal variants use
ensure_vertical_shell_thickness=none; other cases use ensure_all. Inputs carry
no Native outputs; Native admission/effect evidence remains outside git.

Initial Native preflight official and seeded each17/17 exit0:
/tmp/worker3-modifier-top-native-tesis_2u/evidence.json and native-local-effect.json.
