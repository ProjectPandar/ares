# SPEC: explicit brim consumes post-EFC first-layer region outline

Status: **SPEC-REVIEW-REQUEST — docs only, no RED/code authorized**.
Base29f4a49c18139925fec238d31d2fdb6b790fbc30.
Branch worker1/brim-outline-survey-spec. CPU18/coordinator-only integration.

## Independent upstream rewrite boundary

This ports the omitted polygon consumer of libslic3r Brim.cpp, not a new Ares
brim policy. Unlike approved c5bd786ed, it does NOT depend on W4 producer GREEN:
one PrintObject/instance/model-part/region, explicit OuterInner ordering, no
support/raft/skirt; no flattened later-owner wall gate or sandwich branch.
W4-dependent branch remains blocked and untouched. perimeter_append.rs is
strictly off-limits here; no wall ordering/perimeter/group producer changes.

- Brim.cpp:55-62, use_brim_efc_outline: flag AND positive EFC AND positive
  EFC layer count AND no raft. Preserve source gate, not a new rejection.
- Brim.cpp:109-118, get_print_object_bottom_layer_expolygons: append each first
  LayerRegion's post-processed slices, closing_ex at SCALED_EPSILON. These
  already include first-layer modifiers/EFC; do not offset a bounding box or
  apply compensation a second time.
- Brim.cpp:462-468 chooses cached compensated region outline versus ordinary
  lslices;502-513 intersects it with original volume-group slices before area
  generation. Existing421-570/819-878 area/forbidden masks/path materialization
  and ownership remain source-owned.
- GCode.cpp:4456-4540/5325-5340 exports owned first-layer brim before content;
 5748-5775 keeps current-position splitting/clipping. No emitter repair is
  predicted; same-input complete parity will decide, not bounds alone.

## Exact current scaffold and Rust destination

project_slice/gcode_emit/brim.rs::BrimPlan::generate unconditionally clones
PostCompensationPrintObject::as_parts().1.first(): lslices. In
project_slice/compensation.rs:232-278, lslices backups intentionally restore the
pre-EFC outline while region slices retain applied compensation; cached
raw_slices stores those post-compensation region surfaces (Layer::backup_untyped_
slices semantics), despite its name. Body emission already consumes the right
geometry in every observed input. Do not change that producer.

Exact destination: brim.rs::BrimPlan::generate's source-selection step for
**OuterOnly, InnerOnly, OuterAndInner**. Use existing typed ObjectOptions flag /
EFC amount / layer count / raft fields; active source reads bottom region
surfaces already retained in compensation predecessor, performs the cited
per-region closing and source volume-group intersection. Ordinary source stays
unchanged when gate is inactive. Feed selected polygon set into the existing
outer/inner/combined area functions; no independent area algorithm/API.

Possible tiny internal polygon-selection helper under gcode_emit/brim/ is only
justified if needed to keep brim.rs<=400LOC (currently247). Existing cached
post-EFC access is preferable to new state/getters/producer plumbing. The
standalone brims::efc_outline::bounds_or_adjusted compatibility helper is NOT
an acceptable polygon/hole source, fallback or proof of project API parity;
keep it outside this port rather than silently routing through AABB geometry.

No additional Unsupported check, configuration guard, schema change, new
pipeline, feature flag, legacy route or opaque region0 shortcut. BrimEars/
painted/auto are adjacent source boundaries, not implicitly changed by selecting
one shared source for every type; leave their existing source branch untouched.

## Runtime Native / current-main observations

Twelve pure inputs from existing two-holes3MFs in outer-only-brim, inner-only-
brim and outer-inner-brim; immutable source meshes/options/metadata retained
except explicit OuterInner sequence, EFC0 or0.3, outline flag0 or1. Width4/gap0,
EFC layers1/raft0, two Classic walls, no infill/top/bottom/support/tower/skirt.

Exact Native root: /tmp/ares-brim-outline-native-exact-jIJ8p1/.
Current-main CLI root: /tmp/ares-brim-outline-native-s58Yla/ares-cli/.
Native official+seeded serial **24/24** actual exports accepted, current-main
CLI12/12exit0 with real output, using binary freshly built at exact29f4 from
previous discovery and unchanged SHA. No admission/rejection-based fake diff.

Each mode has **3 real positive-XY Brim divergences +9 inactive controls**.
All ordered positive-XY non-Brim role blocks are identical for each same input,
including active cases. Controls have identical ordered positive-XY Brim
blocks too, but this is NOT a formal complete-stream parity gate.

| Active EFC0.3/flag1 | Native brim bbox | Ares brim bbox |
|---|---|---|
| OuterOnly | [93.98,93.98,126.02,126.02] | [93.68,93.68,126.32,126.32] |
| InnerOnly | [98.889,104.889,121.111,115.111] | [99.189,105.189,120.811,114.811] |
| OuterAndInner | [93.98,93.98,126.02,126.02] | [93.68,93.68,126.32,126.32] |

Outer extents shrink0.3; hole-side inner extent expands0.3. Actual ordered
extrusion coordinates differ, not just advertised bbox/config/footer. Ares
uses unchanged ordinary brim while the body already matches Native.

Raw full G-code/results/settings/commands/binary+wrapper/input+source SHA,
stdout/stderr, head/status receipts and per-role witnesses are retained in
both roots. discovery-audit.json in exact root cross-checks original CLI and
new Native, identical pure inputs and repeat Native positive-XY streams.
Actual orca-slicer argv and Native/thread masks18 captured in ALL24 exports,
CLI masks18, zero violations. Initial sampler deduplicated PID/TID/mask across
wrapper exec, omitting Native argv attribution in7official cases; those records
are retained and not affinity gates. External sampler now keys CMD too and
polls20ms; complete fresh Native rerun resolved attribution, not oracle edits.
No build/Native instrumentation/temporary Rust probes or repository code change.

### Exact pure input SHA256

| Input | SHA256 |
|---|---|
| outer-efc0-outline0 | `3c72f5c16d1d7cf4712c6864d6997c75f514ba0b21080d74b5841c3a501986f8` |
| outer-efc0-outline1 | `47e1bb68f59008634693b479b0773bd5d53beb22076a1209ba970aebbed8c29b` |
| outer-efc0p3-outline0 | `2c3b79b13401c5336885b731a94d0e13dba6e74d9ea4b9aecfddfa381544ba41` |
| outer-efc0p3-outline1 | `4726568bd91835c7924be0de47b818c4c9eb6d1e1519484c57b26fa99f57c0cf` |
| inner-efc0-outline0 | `15d328514e72691c1ae89a3bb04190f8b6180628cf721c1308856f4d5e58697b` |
| inner-efc0-outline1 | `30d7199f7a7733a7564a4e6a6113c17fbfbde1917d3d404edbe37f865bb04cbb` |
| inner-efc0p3-outline0 | `76cdc366c107c049bbd60942371eb86279a9bcf28a052befb44d9c4f3969ad44` |
| inner-efc0p3-outline1 | `5fec76301565d5524f0d3696923be31c7d18c03feb90716f3b72e570bd00b0c3` |
| both-efc0-outline0 | `966ffec0a11481dc1ea86183cdae0a29b4515e32bfaa85546fc5fd2fd0cb1f44` |
| both-efc0-outline1 | `449c16c0ef3871617c1e4c3176f79622d9e15468633808361610b68e007b8d07` |
| both-efc0p3-outline0 | `fe3cb45ad5734be625e01cb6704efea88525fcf72dc4db2038dd03817e35ddf5` |
| both-efc0p3-outline1 | `e1c1afa7c18a3b823af0d38da3855b84dff5592f2cf5f63f63a88ff43d22f1b6` |

## Scope, deferrals and approval gates

Included: omitted explicit-type polygon source selection with existing EFC
predicate, polygon holes and post-compensation first-layer geometry. Ordinary
lslices and inactive flag/zero-compensation outputs must stay byte-identical.
Preserve source masks, contact retention, quantization, flow, area ownership
and seam splitting. Do not claim all shapes are proven by a bbox observation.

Deferred: auto/ears/painted snapping, multi-region volume-group ownership,
multiple instances/filaments, support/raft area behavior, layer-count validation
policy (project compensation currently requires positive count; do not relax it
as part of this port), EFC producer/math, general XY compensation, W4 physical
wall ordering, lower-slice identity, per-object skirt topology or GCode policy.
Deferred variants must not be changed as an incidental shared-source effect.

1. Approve docs-only SPEC before separate public Native-first API+CLI RED.
   Twelve exact pure inputs, expected3active orderedFAIL+9controlPASS per mode;
   full existing comparison, all travel/arcs/M73/stats/footer preserved.
   Independent RED review before GREEN. No rejection-as-success/normalization.
2. Source-cited minimal consumer only; no code in compensation producer,
   perimeter_append or ordering; no double EFC or AABB fallback. First raw
   difference drives any source amendment; if a wider producer seam is needed,
   stop and request approval instead of redesigning it.
3. GREEN complete strict same-input Native/API/CLI +raw API/CLI equality,
   extrusion/holes witnesses; raw9controls RED->GREEN unchanged. Existing
   OuterOnly/InnerOnly/OuterAndInner/ears/bed/contact and skirt suites retained.
4. Same final exact hash/clean before+after directed two modes/strict/fmt/WASM,
   CPU18/jobs1/threads2, serial fresh Native caches/artifacts/owned target,
   actual descendant/thread masks and START/END. Obey exclusive freezes;
   no full/workspace without coordinator allocation, no worker push.

Only docs are committed; discovery inputs/Native outputs remain external.
