# Gated role-flow input-only fixtures

Approved whole-block specification: `4701b06a025ab81b20a2ea73becf26a2cb4d36df`,
`docs/architecture/wall-role-flow-ratios-spec.md`. Upstream source pin
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`libslic3r/GCode.cpp:6485-6507` (all eight roles and gated first-layer rule).

These 15 archives are exact copies of the approved Native-discovery inputs.
`manifest.json` freezes full archive/member SHA256, retained settings and
independently observed Native positive-role counts. `geometry-manifest.json`
records the three simple closed/oriented genus-0 models. No Native output is
committed. The initial 0%-sparse attempt is NOT used.

Native evidence at approved spec hash:
`/tmp/ares-w1-gated-flow-spec-reviewed-scope/spec-review-audit.json`:
60 successful Native slices, official/seeded, two independent processes each,
CPU18 actual ELF/all sampled threads, complete generator-only repeat and
cross-mode equality including config/M73/statistics/order.

| Input | Role setting / purpose |
|---|---|
| box-unit | All unit ratios, enabled gate; box control |
| outer075 | outer_wall_flow_ratio=0.75 |
| inner065 | inner_wall_flow_ratio=0.65 |
| sparse070 | sparse_infill_flow_ratio=0.70 |
| solid060 | internal_solid_infill_flow_ratio=0.60 |
| gate-off-all | all non-unit role/first-layer settings retained, gate off |
| first-unit | unit ratios, positive brim and skirt |
| first055-on | first_layer_flow_ratio=0.55, gate on |
| first055-off | first_layer_flow_ratio=0.55, gate off |
| gap-unit | thin-prism unit control |
| gap070 | gap_fill_flow_ratio=0.70 |
| support-unit | T-overhang normal-auto support unit control |
| support070 | support_flow_ratio=0.70 |
| interface065 | support_interface_flow_ratio=0.65 |
| overhang070 | overhang_flow_ratio=0.70 |

All eight role targets have real positive extrusion discrimination versus their
units with unchanged corresponding XYZ/role/layer sequences; first055-on only
changes eligible first-layer moves and preserves positive brim/skirt extrusion.
Both gate-off bodies equal their units, but config settings stay different.
These paired diagnostics are NOT the public parity comparator.

Every public test first runs Native twice, validates retained settings and
activity, compares complete Native repeats with generator identity only, then
requires the validated byte-in API and real CLI to succeed, raw API=CLI, and
unchanged whole ordered generator-only comparisons against the same-input
Native output. No expected-failure assertions or production changes.

Owner RED instruction: role/first-layer-on targets should strictly FAIL;
unit and gate-off controls should PASS in both modes. Any unexpected PASS/FAIL
must STOP and be reported, not normalized, patched or hidden. In particular,
pre-existing sparse/solid behavior may yield an unexpected PASS; that is not
permission to fabricate RED or continue production work. Run per-leg barriers
so later cases do not silently proceed after an unexpected result.
