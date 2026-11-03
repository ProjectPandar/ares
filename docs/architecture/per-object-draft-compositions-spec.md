# SPEC: divergent draft-shield minimum and outer-brim legs

Status: **SPEC-REVIEW-REQUEST — docs only; no RED/code authorized**.
Base: `57fea94989ff362129d0e53da0c35c298822f689`.
Branch: `worker1/draft-compositions-spec`. CPU18; coordinator-only merge/push.
This is a bounded composition of existing libslic3r rewrite slices, not a new
Ares pipeline, extrusion policy or obstacle API.

## Discovery ruling / exact included legs

Only the two independently measured divergent legs are proposed:

A. draft enabled + positive min_skirt_length, **no brim**;
B. draft enabled + explicit OuterOnly brim, **min_skirt_length=0**.

Both include all-loop and single_loop_draft_shield modes, positive skirt loops,
one filament, by-layer, uniform skirt angle. Brim leg is combine_brims=0 and
one printable instance per PrintObject. Far separate and near brim-shared
skirt groups retain their source-owned member identity and per-Z once dispatch.

Enabled/no-brim/min0 inputs are already-admitted controls, not an implementation
goal. Triple draft+positive-minimum+brim has not been discovered/proven and is
**deferred**, not silently admitted through the two-leg composition.

No new guards/errors/fallback. Only narrow the existing draft term:
D && (M || B), where D is non-disabled draft, M is nonzero minimum and B is any
materialized BrimPlan, retains only D && M && B for the deferred triple. New
rejection implies old rejection; no added guard site/category. Preserve all
other terms, resolved.objects.len()>1, and existing singleton/OuterOnly/
combine_brims brim admission verbatim. Ordinary no-draft min+brim stays admitted.
If the scoped boundary needs an additional option policy, stop and ask instead.

## Owning upstream boundary

Vendored OrcaSlicer/src/libslic3r:

- Print.cpp:575-586, has_infinite_skirt/has_skirt: enabled plus positive loops
  extends beyond skirt_height, not beyond the object physical layers.
- Print.cpp:2520-2558: make_brim creates plate-frame brim areas/owner maps
  before _make_skirt; source ordering is not optional.
- Print.cpp:2646-2696, Print::_make_skirt: infinite shield safety height uses
  all object layers; collect object/support occupied contours up to that bound.
- Print.cpp:2703-2778: fixed initial flow and Extruder conversion, local budget
  per append_skirt_loops_for_hull call. The budget counts full **skirt** loop
  length, never brim extrusion, and determines materialized loop count.
- Extruder.hpp:38-41: filament flow ratio/cross-sectional-area conversion.
- Print.cpp:2794-2819: append each object's plate-frame brim-area outer contours
  before the object's occupied convex hull; do not shift twice or use path bounds.
- Print.cpp:2830-2900: fixed-point groups expand by configured nominal
  skirt_distance+skirt_loops*spacing, not minimum-induced extra reach.
  PrintConfig.cpp:5751-5760 documents final loops are not placement validation.
- Print.cpp:2898-2918,2972-2996: independent brim owners when combine=false,
  generate/reverse each group skirt, retain membership; flat skirt is preview.
- GCode.cpp:4260-4331: actual loop entity count, first/later Z scheduling and
  infinite lifetime; GCode.cpp:4391-4452 selects all on first layer or the own
  range's final/innermost loop on later single-loop layers.
- GCode.cpp:4456-4540,5325-5336: group once-per-Z, then each object's first-layer
  independent brim, then its content. Shared skirt does not share brim budget.
- GCode.cpp:5748-5775: current-position loop splitting can change brim stream
  segmentation/seam travel. Never assert min/mode changes leave brim bytes equal.

The positive generation minimum is **not a per-export-layer target**. With
single-loop shield, later layers intentionally emit one innermost loop even
when its emitted filament is far below the configured generation minimum.
Do not top up later layers or re-run the group budget in the emitter.

## Actual two-mode Native and current-main Ares evidence

Root: `${TMP_ROOT}/ares-draft-compositions-spec-2f3uvK/`.
Twelve external pure 3MFs: far/near geometry × control/minimum/brim × all/single.
Source is existing far-brim/near-brim input; only explicit draft, minimum, brim
and single-loop settings vary. All other geometry, source IDs, singleton
instances, transforms and options remain. Width8/gap0/combine0 when brim;
loops2/height1/distance6, uniform 4mm objects/0.2mm physical layers.

Official+seeded ran serially, fresh case caches/artifacts, exact base clean
before/after; **24/24** return_codes/OS exits0. Native inputs/wrapper/binary SHA,
launch, logs, full G-code/result, per-Z witness and aggregate witness-summary
are retained. Actual sampled Native/thread masks18, zero violations. Relative-E
markers and exact input SHA were checked; each layer has real positive XY
skirt movement, not just admitted settings. Every case has20 skirt Zs0.2..4.0.

| Leg (both modes) | Groups far/near per Z | First-layer E mm/group | Second-layer all -> single E mm/group | Brim |
|---|---|---|---|---|
| control min0/no brim | 2 / 2 | 4.90079 each | 4.90078 -> 2.41445 | none |
| A min20/no brim | 2 / 2 | 21.36385 each | 21.36386/21.36388 -> 2.41445 | none |
| B min0/outer brim | 2 / 1 | far7.82392 each; near11.51098 | far7.82392/7.82393 ->3.87567; near11.51097 ->5.71919 | 2 owners only first layer |

All/single first-layer motion is identical within each geometry/leg. Every
later single variant has lower E and strictly smaller X bounds than its full
counterpart. Leg A proves separate group budgets and **no later top-up**;
leg B proves brim-aware grouping (near2 ->1) plus independent first-layer brims
and continued shield lifetime. TYPE blocks are group emissions, not loop counts.
Export clips seams, so emitted-E target comparison is not the budget definition.

Current-main CLI was freshly built at exact base Rust1.99.0/jobs1/CPU18,
CARGO_INCREMENTAL=0, explicit owned target reuse; build exit0, head/status
receipts and binary SHA retained. Twelve CLI probes (sampled actual masks18):
4 controls exit0; **all8 A/B inputs exit1** with
Error: UnsupportedProjectFeature("skirt_type per-object"), no G-code. Thus both
proposed legs demonstrably diverge, rather than being guessed from the guard.
Standalone discovery is not committed public API+CLI RED, strict parity or full
coverage; no test/production changes occurred.

### Exact pure input SHA256

| Input | SHA256 |
|---|---|
| far-control-all | `b5e4d5fd78a312b3f5a99e7658c1f70105c80f6e0f9b0f0121d22787e24c9cd1` |
| far-control-single | `28c5cfd48107ed37c1f093633db210dc368b36d43f87b9350421c80a42d6b56b` |
| far-minimum-all | `a73ce749f8b140e648903d50861157c1c7b97d8acf849ea1681f99cfadf28ba3` |
| far-minimum-single | `fdc5a82c08f7c3029c9246a64acfe145cf6102f94e23d8b84655d746ffb3820d` |
| far-brim-all | `5cea8117b5fe570bc557411f59569e2d5ce61e8efe999a9474d1d6b5c601e1b2` |
| far-brim-single | `6caf05d465704a99254fd92798e8f5f67a0d999bc669a54df8d36b3475e42a14` |
| near-control-all | `e5bb4e76245a7ae70f71f36cbe644eed8368c5979df5d9df8af5e7fd9d72c5b0` |
| near-control-single | `738c0397569a74806b7f8d9a410449c45258e82fefc1f07c20026d6d92f7c414` |
| near-minimum-all | `2693142ce998bdf99b9134a3d4afef7f2716d06e692823a1aa916a6801c03e1d` |
| near-minimum-single | `7d7c63c8371a49fd3fbc44e39f2dc539258e19e2d6052eeae3b738f29ebfc17c` |
| near-brim-all | `b639ebcde6d14cf69344c6c9fcaa15888b70104c78c773838a3a0f88af9b4a7c` |
| near-brim-single | `e0426c06978f89d026dba478a9eae78fed1a85f8b33fd14bf1d4aaa1ba7b06ed` |

## Exact Rust destination / compatibility-shell treatment

Under crates/ares-core/src/project_slice/gcode_emit/:

1. skirt/generate.rs: only the existing draft && (nonzero-min || BrimPlan)
   rejection term widens for measured A/B. Independent brim and other deferred
   admission stays byte-identical; deferred triple remains inside that term.
2. skirt/loop_generation.rs::make_loops / generate per-group call: existing
   positive-only local budget ports Print.cpp. Audit actual full stream, do
   not add another budget, count brim E, or recompute per layer/member/copy.
3. skirt/occupied.rs::items and brim.rs area-derived covered_hull: existing
   infinite-height and brim plate-frame producer mapping is consumed, not
   replaced by raw object bounds or an invented obstacle abstraction.
4. skirt/member_groups.rs, SkirtPlan own ranges and layers/object_skirt.rs:
   retain nominal fixed-point envelopes, identity, advancing-Z done state,
   physical-height flow delta and own-group innermost suffix already merged.
5. layers/tool_entry.rs / brim_emit.rs / brim/emit.rs / motion/skirt_brim.rs:
   retain skirt->owned brim->content sequence, once first-layer brim ownership
   and current-position split/clipping. Do not assume constant brim bytes
   across loop-mode or budget changes; compare same exact input only.

These remain temporary compatibility shells where they differ from cited
Print/GCode behavior. First raw difference determines the smallest source-
cited interaction repair; no speculative motion/producer refactor. Stop/report
identity, centering, layer producer or wider-boundary problems to the owner.

## Deferred / approval and exit gates

Deferred: triple draft+positive-min+brim, non-singleton brim ownership,
combine_brims, auto/inner/ears/mixed brim policies, support/support-brim/raft,
tower, sequential/multifilament ordering, nonuniform angles, zero-loop draft,
new placement or variable-height producer logic. No new guard enforcement
beyond strictly narrowing the current term and preserving existing policies.

1. SPEC approval before any RED. Independently commit twelve exact pure inputs
   and public Native-first API+CLI success tests; expected4controlsPASS+
   8A/BguardFAIL each mode. Review RED before GREEN; no rejection-as-success.
2. Fresh two-mode gates preserve full streams, exact input SHA, per-Z real
   motion, generation-budget/inner-loop facts and first-layer brim ownership.
   GREEN requires existing strict ordered Native/API/CLI comparisons with only
   established generator metadata treatment, plus raw API/CLI equality.
3. Preserve control raw bytes RED->GREEN, all previous draft12/min+brim4/min8/
   brim4/group4/combined/single/zero-loop controls and relevant option domains.
   No fixture/assertion deletion/relaxation or classifier/comparator edits.
4. Exact submitted HEAD/clean status before/after each CPU18 serial leg/gate,
   jobs1/threads2, fresh Native caches/artifacts, explicit owned target policy,
   actual descendant/thread masks, START/END inbox; obey full freezes.
5. Source-cited minimal code only after reviewed RED; <=400 LOC, mechanical
   splits before growth. Final-hash strict/fmt/WASM; full/workspace windows
   separately authorized. No worker push. Only docs-only SPEC is submitted now.
