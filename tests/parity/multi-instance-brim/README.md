# Multi-instance PrintObject brim collection inputs

Input-only 3MFs; no Native outputs committed. Source:
Brim.cpp:36-45,573-581,914-977; Print.cpp:2794-2816;
GCode.cpp:4486-4531. SPEC: docs/architecture/multi-instance-brim-collection-spec.md.

Two owners, three instances: object2 at (80,110,2)/(140,110,2), object3 at
(110,110,2); combined skirt, ordinary combine_brims0, ByLayer, one filament,
outer_only brim8/gap0, no support/raft/tower.

- target: with-skirt genuinely failing whole-output witness.
- no-skirt: only skirt_loops2→0, SECOND failing witness, not a control.
- control: only brim_width8→0, strict passing whole-output control.

SHA256 manifest; non-config ZIP members identical, one named config field
changed for each variant. Tests generate Native FIRST at runtime, check effective
settings/positive extrusion/brim/skirt anchors, then validated API and real CLI.
Every case asserts complete ordered Native equality using the unchanged
generator-only comparator and raw API==CLI. No M73 removal, divergence-as-success,
tolerance, coordinate snap or guard widening. A faithful source port leaving any
whole-output divergence must STOP; estimator work is outside this slice.
