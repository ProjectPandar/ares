# Z contouring: remove the live zaa_enabled capability gap

Status: SPEC-REVIEW-REQUEST, Native preflight complete; no Rust/test changes.
Base: de1211be5067d0eb6f709602a1299a0854c89145.
Upstream OrcaSlicer:8500fcdccaa10b5099ac20d252af3a7c560046f1.

## Actual gap and Native-first evidence

project_slice/capabilities.rs:38-42 rejects zaa_enabled whenever the model
cannot be proven an inactive axis-aligned box. A sloped-roof wedge fails
is_axis_aligned_box: its roof vertices include an intermediate Z value and
its roof normal has both horizontal and vertical components. Global
zaa_enabled=1 reaches this live guard through resolved model-part regions.
This is a slicing capability gap, not a missing matrix receipt.

Before this SPEC, Native official onCPU13 and seeded onCPU14 each sliced
that wedge with zaa_enabled=0 and1, then repeated both inputs with fresh
cache/root/target. All8 invocations exited0 with CONFIG_BLOCK_END. Their
exact same-input results are:

| Input / either mode, both repeats | Active output evidence |
|---|---|
| disabled control | 0 positive-extrusion XYZ commands;142 Top surface moves |
| enabled witness | 418 positive-extrusion XYZ commands;648 Top surface moves;69 distinct extrusion Z values |

Enabled XYZ roles are Inner wall, Outer wall and Top surface. The requested
flag is echoed as0/1 in the Native config footer. After removing only the
existing generator/date header, all four disabled outputs have normalized
SHA f6b280ea634c3ba6f2c955a25d2154c6a74fde71cb1255dfbdc074086fc41c60;
all four enabled outputs have SHA
b3f19dabe585affc963b665bb6ac00760c7e81ec34176606d5f2462fb4ca9533.
This repeat check is evidence, not a new comparator rule.

Inputs: existing admitted cube-derived single-part3MF, retain its ordinary
printer/filament settings and instance transform; change its roof to
z=x/5 while retaining bottom z=-5 and original triangle topology. Shared
settings are classic walls, no brim/support, no arcs. Each pair differs only
in zaa_enabled. The transformed model has a sloped upper surface and a flat
bed-contact bottom. This is a small triangular mesh, not a synthetic
unsupported config or an algorithm approximation.

Retained external evidence (not committed Native products):

- /tmp/worker2-zaa-native-official.bj5xtpb3
- /tmp/worker2-zaa-native-seeded.c8pq5425
- /tmp/worker2-zaa-native-evidence.json
- /tmp/worker2-zaa-native-preflight.py (driver; offline decimal-token recount
  in evidence includes the Native formatter's leading-dot numbers).

Every run retains input3MF, stdout/stderr, raw G-code and XYZ evidence;
producer.json retains exact wrapper/ELF/source hashes and base commit.
No Ares public RED has been run yet: rejection is established by the current
source predicate and will be verified in its independent RED commit.

## Owning libslic3r boundary

| Source file / function / lines | Required behavior |
|---|---|
| PrintConfig.cpp::PrintConfigDef:4378-4413 | Enable flag, min-height, wall-angle and fill-direction options/defaults. |
| PrintObjectSlice.cpp::compute_slice_z:24-47 | First layer midpoint; subsequent enabled layers use lower bound plus minimum enabled-region zaa_min_z. Existing Ares layers.rs:245-280 already carries this concept; verify rather than replace it. |
| Print.cpp::Print::process:2331-2336 | Contouring occurs after fill/ironing preparation. |
| PrintObject.cpp::need_z_contouring/contour_z:748-804 | Region activation; transformed raw model mesh, centered object frame/ground offset; skip first layer. |
| SLA/IndexedMesh.cpp::normal_by_face_id/query_ray_hit:139-171 | Nearest upward hit, normalized triangle normal and float hit-distance carrier; retain no-hit semantics. Acceleration is part of the boundary. |
| ContourZ.cpp::contour_extrusion_path:18-159 | Eligible roles;0.1mm sampling; ray distance, roof/wall-angle adjustment, height clamps, perimeter upward exclusion, collinear simplification; replace polyline and mark z_contoured only when active. |
| ContourZ.cpp::contour_extrusion_entity/Layer::make_contour_z:162-245 | Traverse paths, multipaths, loops and collections; filter per-region eligible fill/perimeter roles. |
| GCode.cpp::_extrude:6374-6400,6992-7030,7220-7237 | 3D start travel and nominal-Z restoration; XYZ extrusion; per-point height-based E scaling; contoured paths bypass ordinary arc fitting. |

Ares's capability rejection is temporary scaffolding around an unported
ContourZ/ExtrusionPath boundary. Delete it only together with actual path
contouring and G-code materialization; deleting the guard alone is not GREEN.

## Rust destination and included behavior

- New project_slice/contour_z module, split into mesh-query, path and entity
  traversal files as needed (<400LOC each). It ports the source algorithms,
  not wedge-specific interpolation or a new Ares contouring pipeline.
- Reuse current resolved model mesh/transform and layer/region data. Build
  the upstream-shaped indexed ray-query representation once per print
  object; do not rescan every triangle for every0.1mm point. Follow nearest
  hit and normal semantics, including source numeric carriers and scaling.
- Reuse existing layers.rs scheduling. Wire the contour stage after completed
  fill/ironing entities and before consumers discard per-point Z. Confirm
  source order against current slice_source/entity preparation before PLAN.
- Extend perimeters/classic/materialize/types.rs::ExtrusionPath and fill
  entity carriers with the upstream z_contoured meaning, preserving XY/Z
  together through ordering, seam handling and simplification. Existing
  Point3/Polyline3 is a compatibility carrier, not a separate path design.
- Extend gcode_emit/motion/path and its start-travel/extrusion consumers to
  carry contour Z and source E scaling, including return to nominal Z for
  following ordinary paths. Do not flatten XYZ into today's XY iterator.
- Remove capabilities.rs zaa rejection and only helper/imports proven orphaned
  by that deletion, once the new stage makes the admitted RED strictly PASS.

Included algorithm roles: top solid fill, ironing, inner/outer perimeter,
and their existing collection forms, with source role exclusions intact.
First RED uses classic constant-width walls, default minZ0.05, wall-angle35,
ordinary layers and default fill-direction bool; no broad option-domain
coverage is claimed by that finite fixture.

Deferred follow-up witnesses: Arachne variable-width interactions, modifier
boundaries, multiparts/multiple instances, nondefault minZ/angle,
zaa_dont_alternate_fill_direction, ironing-specific fixtures, scarf/arc
interactions and arbitrary topology. Their algorithms must not be replaced
by new Unsupported guards, shape whitelists, silent skips or fallback.
If implementation requires a larger boundary than above, stop for revised
PLAN; do not claim generic coverage or silently change adjacent algorithms.
W3 modifier logic, W1 skirt and W4 counterbore are untouched.

## Independent public RED and review sequence

After SPEC approval, commit a public E2E test first using the witnessed same
input bytes. Assert Native acceptance/footer and active XYZ roles, then call
public Ares API and CLI and require existing strict whole-output PASS.
Expected current RED is UnsupportedProjectFeature("zaa_enabled") / CLI
rejection for enabled wedge. Disabled wedge is the ordinary output control;
if it already mismatches, STOP and report the separate gap before repair.
Keep inputs, API/CLI results and all streams as repeatable external artifacts.
Native cannot be represented by an expected rejection for this live feature.

Report RED separately, then obtain repair PLAN approval for stage/carrier
changes before production. Existing unit tests asserting the obsolete
capability rejection require an exact-diff retirement request; do not change
assertions without approval. GREEN must preserve disabled control bytes,
strictly match both Native modes, and retain the enabled XYZ/height/E witness.
No comparator normalization/weakening or new suppression is permitted.

Run directed only onCPU13 official/CPU14 seeded, outer taskset, jobs2/threads2,
fresh root/cache/target and START/END inbox entries. After repair: directed
both modes plus strict clippy/fmt/wasm on the exact hash, REVIEW-REQUEST;
workspace/full/CI only after coordinator allocation. No push by worker2.
