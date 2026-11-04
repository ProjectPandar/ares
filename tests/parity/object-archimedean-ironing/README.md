# Object Archimedean Chords ironing: frozen Native-active RED

Owner: OrcaSlicer pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`Fill.cpp:1611-1640,1705-1731`, `FillBase.cpp:61`,
`FillPlanePath.cpp:69-200`. This is object ironing, not support/raft.

`manifest.json` records full input SHA-256 and every archive-member SHA.
Input-only closed 10 x 10 x 1 mm box, 8 vertices / 12 oriented triangles,
five .2 mm model layers, `top_shell_layers=1`. No support, raft, modifiers,
tower, skirt or brim. All five model layers must extrude; topmost ironing
must occur only on the final layer, not all five layers.

- `archimedean-active`: topmost / archimedeanchords;
  `7a900b9b6a31a9ae7731dca238c601c39d81c22419b7ff04979d8a155a96c326`.
- `rectilinear-control`: changes only `ironing_pattern` to rectilinear;
  `74f447fffe933f8d5540426e8fea625894c9c63ee5fbce679861e9fcb94a5cce`.
- `archimedean-disabled-control`: changes only `ironing_type` to no ironing;
  `4862a3f50cbbe63785962306277417ec2ff32391084ddc446d52a7a54ba1c0a6`.

Tests run Native first and assert real positive extrusion on all five model
layers, plus final-layer positive Ironing for target/pattern control and
none for disabled. Native's retained configuration must name the selected
pattern and ironing type. Only then invoke validated public API and actual
CLI. Require success, raw API=CLI and unchanged complete ordered Native
comparison (`golden::compare_ordered_bytes_generator_only`) for both.
The target is expected to genuinely FAIL at existing
`UnsupportedProjectFeature("ironing_pattern")`; Unsupported is NOT a passing
assertion. Both single-field controls must wholePASS in both modes.

Discovery's defined-behavior audit checked the non-null factory, initialized
FillParams (role 0 short-circuits optional config reads), solid centered
PlanePath domain and positive bounded Archimedean acos/spiral domain. Native
success alone was not treated as a UB-safety proof.

No production code or Native outputs are committed here. Before adding the Archimedean dispatch,
mechanically split existing 400-LOC `ironing.rs` (e.g. move PlanePath dispatch
to `ironing/plane_path.rs`), preserving behavior. Reuse the single existing
`fill::plane_path` implementation; do not copy a generator or broaden other
patterns. Preserve controls' raw API/CLI bytes RED to GREEN. Any residual
whole-output mismatch means STOP, not M73/timing repair or filtering.
