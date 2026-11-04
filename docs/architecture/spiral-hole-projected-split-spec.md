# Spiral hole projected-split rewrite SPEC

Sequence: SPEC → tests-only RED → minimal GREEN.
Built on frozen reset-comment `83e2e4f090a54f64fd17e4895db735f7c8f8057f`.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Upstream-first boundary

`libslic3r/GCode.cpp::GCode::extrude_loop:5754–5775` contains two independent
conditions: configured spiral orientation excludes holes, while configured
spiral `loop.split_at(last_pos, false)` applies to every perimeter loop,
including holes. `ExtrusionEntity.cpp:230–320` owns projected closest-point
selection, endpoint snapping and rotation. `PerimeterGenerator.cpp:124,528`
assigns the hole loop role. This is a libslic3r caller-condition rewrite,
not an independently designed Ares pipeline or shape-specific repair.

Rust destination: `ares-core/src/project_slice/gcode_emit/motion/perimeter.rs::
emit_perimeter`. Remove only the hole exclusion from its configured-spiral
projected-split branch. Reuse `seam_placement::place_nearest_projection` and
`split::split_at_projected(..., false, ...)`; preserve orientation handling,
nonspiral seam selection, Path/MultiPath dispatch and clipping/wipe algorithms.
The current extra role condition is a temporary incompatible caller shell,
not an upstream requirement. Do not extend it with a new fallback.

## Witness and controls

Use the actual official-Native exported Prusa MK4 0.4 / default
`0.20mm Standard @MK4` / `Prusa Generic PLA @MK4` on unchanged upstream
`tests/data/cube_with_hole.obj`. Inputs only are committed; no Native product.

- Stock-hole control SHA256:
  `4c4ceaf15b083f1882c29ab0dc867ae62099abd3d3c13c30c8b6f6c2416766fa`.
- Spiral-hole target SHA256:
  `7737bc4991c554b645ceed6fe819462cfb1ece3ad2b6cfdf9095dacd78dbde27`.
- Preserve existing spiral CW-rectangle regression input/test, SHA256:
  `94a5b7e51d536faae94e4900b17f4eadf92fb0dea4c7c3d3f2c53c7f4231a48a`.

Discovery on main1a57f677f: target Native replay raw equal; API/CLI succeed
and are raw equal but both first differ at comparator byte2629, line136,
column9: Native `G1 X119.818 Y99.818`, Ares `G1 X119.76 Y99.957`.
Stock, ironing-on and arc-on discovery controls passed full output.
These are discovery observations, not formal RED evidence on this stack.

One requested process key (`spiral_mode`) changes before export. Native's
`PrintConfig.cpp:8567–8582` normalization also changes wall_loops3→1,
top_shell_layers4→0, sparse_infill_density15%→0%, and both layer-change
retraction vectors to0. Retain these actual exports, not a false sole-key
archive claim. GUI `ConfigManipulation.cpp:345–378` supplies a source-
constructive correction route matching target settings; no live GUI-save claim.

## RED then GREEN qualification

Commit SPEC/ARD before tests; commit tests-only RED before any production edit.
Use unchanged `runner::native_stream_case::qualify/check` and `NativeCase`.
Add no helper, scanner, activity gate or private oracle. A Native-only preflight
qualifies all three cases twice in both official/seeded modes before public
calls:12 successful slices, six stable replay pairs. Public tests must require
complete output equality without expected-failure conversion. RED must retain
target API/CLI differences with both successful and raw equal, and passing
stock/CW controls. Both mode failures must be real test failures.

GREEN removes only the role exclusion and its newly unused import. All cases
in both modes must pass Native/API/CLI complete comparisons and raw API==CLI.
Run cargo fmt, workspace/all-target clippy, workspace checks and browser WASM
check. Preserve binaries, failed attempts and raw evidence.
Do not amend the frozen reset commits.

Included: configured spiral projected split for perimeter holes, including
bottom layers. Deferred: orientation redesign, SpiralVase filter arithmetic,
perimeter geometry, seam ranking/scarf/random, travel/wipe, brim and processor
caches. Any residual whole-file mismatch stops expansion and must be reported.
See [ARD](ard-spiral-hole-projected-split.md).
