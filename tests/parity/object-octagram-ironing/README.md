# Object OctagramSpiral ironing: standalone frozen RED

Conditionally approved proposal:
`dce718354eca59d3e4c69fbc06693d1af1f5411c`, Octagram ONLY.
RED stacks directly on approved Archimedean GREEN
`05bf7628cf58ecee5227010e93241d7f55e1f0d5`; no approved chain is rebased.
ZigZag/CrossZag/LockedZag are NOT covered or admitted by these tests.

Owner: upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`Fill.cpp:1611-1640,1705-1731`, `FillBase.cpp:63`,
`FillPlanePath.cpp:69-164,278-315`. Destination is existing OBJECT
`fill_entities/ironing.rs::append` and its already split `ironing/plane_path.rs`,
not support ironing or raft. The source solid branch uses centered snug
surface bounds and the finite bounded Octagram generator; FillParams role 0
short-circuits the optional print_object_config calibration read.

Input-only closed 10 x 10 x 1 mm box, 8 vertices / 12 oriented triangles,
five .2 mm model layers, top_shell_layers=1. No modifiers/support/raft/tower/
skirt/brim. ALL five model layers print, but topmost ironing occurs only on
the last layer. Each input SHA and every archive-member SHA is in the manifest.

- `octagramspiral-active`: topmost / octagramspiral;
  `1c9aa16cd1644701454a5dd2cc077169be95b0baf8d86c514c75e97cff2166a6`.
- `rectilinear-control`: changes ONLY ironing_pattern to rectilinear;
  `74f447fffe933f8d5540426e8fea625894c9c63ee5fbce679861e9fcb94a5cce`.
- `octagramspiral-disabled-control`: changes ONLY ironing_type to no ironing;
  `cffd3843fe9ac921791c188a6b3d1e45eb766cd440a2bfbe10b05de2b6c3ab28`.

Native-first tests prove positive model extrusion on all five layers and
positive last-layer Ironing for target/pattern control (none for disabled),
with retained selected pattern/type config. Each leg then runs Native again
in a separate output directory, reasserts activity and compares COMPLETE
Native streams using unchanged `golden::normalize_one_generator_line` with
Orca kind on BOTH sides, plus `first_difference`. Only generator identity
normalization is allowed, not body/config/M73/order filtering.

Only after these Native gates do tests call validated public API and actual
CLI, requiring SUCCESS, raw API=CLI and unchanged full ordered Native/API and
Native/CLI comparison (`golden::compare_ordered_bytes_generator_only`). The
active target must genuinely FAIL at UnsupportedProjectFeature(ironing_pattern),
not pass by asserting rejection; both controls must wholePASS in both modes.
Native outputs are runtime artifacts, not committed expectations.

Independent RED approval precedes GREEN. GREEN must reuse the already split
shared PlanePath helper with `PlanePathPattern::OctagramSpiral` and the SAME
parameters as approved Hilbert/Archimedean. No second generator, bbox source,
new guard, parser expansion or support/raft change. Preserve both controls'
raw API/CLI bytes from RED to GREEN and prior approved ironing outputs.
Residual whole-output mismatch means STOP/report, not M73/timing repair,
tolerance, snapping, filtering or fallback. CPU18, fresh serial official and
seeded roots/caches, exact clean hash and targeted gates only.

Discovery repeat/activity evidence:
`/tmp/ares-w1-object-ironing-remaining-discovery-8f6ce1660/discovery-report.json`.
The separate zag work remains blocked on discriminating Native toolpath-body
geometry after excluding the config block, then a separate proposal/RED;
its bbox must independently follow the real object lifecycle in
`Fill.cpp::make_ironing`. W3 confirmed support/raft's ±1mm dummy bbox_object
(`SupportCommon.cpp:1456`) is a DIFFERENT source: no alignment is required,
and it must not replace the real object bbox or create a second object-bbox
producer.
