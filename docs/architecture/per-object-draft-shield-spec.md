# SPEC: per-object draft shield lifetime and group-local single loop

Status: **SPEC-REVIEW-REQUEST — no production/tests authorized**.
Base: `104d8497097dc7239ad63f7a69f3b418b699b3e8`.
Branch: `worker1/per-object-draft-shield-spec`.
The approved min+brim chain remains untouched on its stack-7 candidate branch;
no push. This is a source-cited libslic3r rewrite slice, not a new Ares pipeline.

## Non-negotiable scope

Include enabled draft_shield with positive skirt loops, one filament, by-layer,
no brim/support/raft/tower, min_skirt_length=0, uniform skirt angle. Cover the
existing disjoint/overlap/transitive/copies PrintObject membership model and
both single_loop_draft_shield=false and true. Disabled controls remain unchanged.

Enabled shield is infinite **relative to skirt_height**, not an extra object
or a forever-running emitter: emit at the participating object's physical
layers, once per group/Z. Preserve group membership and fixed-point envelopes.
First layer emits all requested loops; later single-loop layers emit that
**group's innermost materialized loop**, not the last loop of the flat global
preview mirror. Do not change combined/single-object loop selection.

Only narrow the existing draft_shield rejection term for the no-brim/min0
slice; keep positive-minimum/shield and brim/shield combinations deferred in
that existing term. Keep independent brim, support, raft, tower, sequence and
filament terms and original resolved.objects.len()>1 predicate unchanged.
No new guard/error, combined fallback, obstacle model or height/placement policy.

No code/tests before SPEC approval. Independent public RED commit and review
precede GREEN. No Native-output commits, assertion weakening, classifier,
verify_value or comparator changes. Changed files <=400 LOC; split mechanically
before growth beyond that bound. Coordinator alone merges/pushes.

## Owning upstream boundary

Vendored OrcaSlicer/src/libslic3r:

- Print.cpp:575-586, Print::has_infinite_skirt / has_skirt: enabled draft plus
  positive loops is infinite skirt; finite skirt comes from skirt_height.
- Print.cpp:2646-2696, Print::_make_skirt: infinite case takes all object
  layers, determines safety hull height, collects object/support occupied
  geometry. Consume existing producer coordinates, do not redesign slicing.
- Print.cpp:2703-2778: fixed initial skirt flow/spacing, outward loop creation
  and occupied hull; positive-minimum budget exists but is deferred with shield.
- Print.cpp:2784-2900,2972-2996: stable PrintObject/copy ownership, nominal
  fixed-point envelope merging, per-group reversed loops; flat m_skirt is only
  a preview/extents mirror, not the source of group-local loop selection.
- GCode.cpp:4260-4299: actual skirt entity count and first-layer once scheduling.
- GCode.cpp:4300-4331: other-layer schedule admits infinite skirt despite
  configured skirt_height, requires advancing print_z and updates group done.
- GCode.cpp:4391-4452, generate_skirt: flow height from adjacent emitted Z;
  all loops on first layer, later single-loop starts at loops.second-1, i.e.
  the innermost loop after reversal. Seam uses current nozzle on later layers.
- GCode.cpp:4456-4484, generate_object_skirt_group: select group by object ID
  and use m_skirt_group_done[group_idx].
- GCode.cpp:5325-5336: once-before-first-member normal-pass dispatch.

No draft-specific UI, new geometry producer, or new Ares pipeline is included.

## Measured Native admission / effect and current Ares difference

Evidence: `/tmp/ares-draft-shield-native-nF2vkY/`.
Exact base and empty before/after git status are recorded. Twelve pure external
inputs retain original four geometry fixtures, changing only draft_shield and
single_loop_draft_shield. Skirt height1/loops2/distance6; no brim/min0; uniform
4mm objects with physical 0.2mm layers. Each mode/case has fresh cache/artifacts,
input, wrapper/native SHA, launch, complete output/result/log and affinity data.

Official and seeded ran serially at CPU18: **24/24** OS exits and result return
codes 0, actual sampled Native/thread masks18, zero violations. witness.json
retains per-Z groups, positive XY counts, bounds and relative-E sums;
witness-summary.json aggregates them. This is real extrusion/lifetime proof,
not an exported setting or a successful no-skirt file.

| Case geometry | Groups per skirt Z | Disabled Z layers | Enabled full/single Z layers | Second-layer E/group, full -> single |
|---|---:|---:|---:|---|
| disjoint | 2 | 1 (Z0.2) | 20 (Z0.2..4.0), both | 4.90078 -> 2.41445, each |
| overlap | 1 | 1 | 20, both | 7.11316 -> 3.52064 |
| transitive | 1 | 1 | 20, both | 9.57134 -> 4.74973 |
| copies | 1 | 1 | 20, both | 12.27532 -> 6.10172 |

Both modes have identical witnesses. Full and single variants have identical
first-layer group motion (all two loops). Every later single-loop group has
less positive extrusion and a strictly smaller X envelope than its full-loop
counterpart, proving **inner**, not outer or global-last, selection. Shared
members/copies do not multiply emissions. All comparisons are motion witnesses,
not a claim that the full streams have identical config footer bytes.

Current-main CLI was freshly built at exact base with Rust1.99.0/jobs1,
CARGO_INCREMENTAL=0, CPU18 and explicit owned target reuse; exit0, head/status
receipts and binary SHA retained. Twelve public CLI probes: four disabled
controls exit0 and emit skirts, eight enabled inputs exit1 with
Error: UnsupportedProjectFeature("skirt_type per-object"), no output G-code.
Actual sampled CLI masks18; cli-witness-summary.json and per-case stdout/stderr,
exit, launch are retained. This proves an actual current Ares difference;
public API+CLI parity RED is **not yet committed or executed**.

### Exact pure input SHA256

| Input | SHA256 |
|---|---|
| disjoint-disabled | `da42dee8873f9a03eaf548182f4587215efccc8308c6b43067a0d3fbda29531c` |
| disjoint-all | `a987113b20299eb18dd08e35b85d25d4ff2f30a99d54f99d294ba9fefd70cfa5` |
| disjoint-single | `b48a6bec6cd6490e63847b85dfb71e170c854ba180e695b68bd633849ee1cc41` |
| overlap-disabled | `676410991207a1de90b3e9b513ede996fe793bb4b351adbfdc29277a21cc6bf9` |
| overlap-all | `e22cb2cd1aeb818c8b7a961aa3bc0dc49d7c932574d5c2f45dd6cbd55697a994` |
| overlap-single | `857fb5aa281e59a3a3cfeb98d0f8ed02c88d9bddf51a2bbe03c38dd6af9cc6b8` |
| transitive-disabled | `2a44f118263d5b01e5d8f7b309099ff81e4ac6f7a96e44fa5f1d5474ca8c0bed` |
| transitive-all | `c93c292c82e749ce644c52362553abd19823cadd42ea0dc0606738a30e16fdca` |
| transitive-single | `e30b5535900856278a22ac7c3a8b23b2c3992ddc3846333c52eea61d5e61f648` |
| copies-disabled | `de1d6f864030ed371063b79eb87911760680d47345ccd4f60bd559e6f6acaa5e` |
| copies-all | `922af8a08aa5d7904056f4bfc215c98012aba743e761ab26bb13faec703fd16f` |
| copies-single | `4f22bed5cc75c1ad6a230e4544941a78594d0c95cd80f504308ea861792b3565` |

## Exact Rust destination / remaining source gap

Under crates/ares-core/src/project_slice/gcode_emit/:

1. skirt/generate.rs: existing draft_shield!=Disabled term causes rejection.
   Infinite layer_limit=usize::MAX already represents source lifetime; reuse
   it, do not add another draft timeline. Narrow only its deferred predicate.
2. skirt/occupied.rs::items: already consumes all participating object layers
   when layer_limit is infinite. Reuse PrintObject/copy identities and frame.
3. skirt/member_groups.rs, layers/object_skirt.rs: keep per-group advancing-Z
   done state and physical-layer flow delta, not a single global shield counter.
4. skirt.rs::emit_group currently passes the complete group's loop range into
   emit_loops on every layer. The single-loop flag is consulted only by the
   flat loops_for_layer route. This is a source-proven grouped-path gap hidden
   behind admission: after guard narrowing, true must select a suffix **within
   that group's range**, never self.loops.last() across groups.
5. layers/skirt_emit.rs / skirt_schedule.rs are compatibility shells around
   upstream lifetime allocation; retain combined/single-object controls.
   Group export uses actual source-owned range, flow and current-position seam;
   only first raw difference justifies additional minimal source-cited repair.

Existing scaffolding is temporary where it diverges from these Print/GCode
functions, not an independent draft design. Do not touch W3 identity/mesh or
unrelated motion. Stop and request a scope decision for producer defects or
any inability to narrow the existing term without a new option policy.

## Included / deferred and post-approval gates

Included: positive-loop enabled lifetime past skirt_height, both loop modes,
first-layer all-loop behavior, later **group-local innermost** selection,
existing group/copy ownership and once-Z scheduling, raw complete export.
Deferred: shield with positive minimum or brim (including pending approved
min+brim composition), support/support-brim/raft, tower obstacles, sequential,
multifilament ordering, nonuniform angles, zero-loop draft interactions,
new variable-height producer/placement logic. Ordinary no-shield min/brim
behavior and all other accepted cases remain unchanged.

1. Before public RED, rebase onto coordinator-broadcast main containing pending
   approved slices; audit source conflict/range identity and fresh witnesses.
   Do not rebase early merely because stack-7 is a candidate.
2. After SPEC approval, commit twelve exact pure inputs/hashes and public API+
   CLI success/strict full ordered Native comparisons plus raw API/CLI equality.
   Native-first fresh two-mode evidence must reproduce per-Z motion and inner
   loop selection. Expected baseline: 4 disabled PASS + 8 enabled guard FAIL
   per mode. Review RED separately; standalone CLI probes are not that RED.
3. GREEN must preserve all fixtures/assertions, old controls' raw bytes,
   first-layer loop set, enabled lifetime and one innermost loop per group on
   later single-loop layers. No admission-only success or normalization.
4. Retain prior min+brim4 after merge, min8, brim4, group4, combined/single/
   zero-loop controls and relevant option domains. Report first raw difference.
5. CPU18 serial modes/jobs1/threads2, fresh Native cache/artifacts, explicit
   owned target policy; exact submitted HEAD and clean status before/after
   each leg/gate, actual descendant/thread masks, inbox START/END. Freeze for
   exclusive full. Final strict/fmt/WASM; full/workspace require separate
   coordinator windows. No worker pushes. Only this SPEC is submitted now.
