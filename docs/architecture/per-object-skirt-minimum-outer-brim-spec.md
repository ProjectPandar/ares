# SPEC: positive skirt minimum length with explicit outer brim

Status: **SPEC-REVIEW-REQUEST — no production/test changes authorized**.
Base: `104d8497097dc7239ad63f7a69f3b418b699b3e8`.
Branch: `worker1/skirt-min-brim-spec`.
This composes two already source-owned `libslic3r` slices; it is not a new
Ares pipeline or independently designed extrusion policy.

## Non-negotiable boundary

Include positive-loop per-object skirts with positive min_skirt_length and
explicit `outer_only` brim, positive brim width, `combine_brims=0`, one
printable instance per PrintObject, simple solid first-layer islands, one
filament, by-layer printing, uniform skirt angle. Far separate groups and
near shared skirt group must retain independent brim ownership.

Only narrow/remove the existing positive-minimum + any-BrimPlan rejection
term in `skirt/generate.rs`. Its independent existing brim predicate still
rejects combined brims, non-OuterOnly plans and non-singleton PrintObjects;
retain that predicate, `resolved.objects.len()>1`, and every other deferred
condition unchanged. No new guard/error, fallback, global minimum budget,
combined-skirt replacement, new collision rule or option policy.

No code/tests before SPEC approval. Independently committed public RED and
review precede GREEN; fixtures are pure inputs. No Native output commits,
assertion relaxation, geometry normalization, verify_value/classification or
comparator changes. Changed files <=400 LOC; mechanical splits precede
semantic growth beyond that bound. Coordinator alone merges/pushes.

## Upstream owning source boundary

Vendored `OrcaSlicer/src/libslic3r`:

- `Print.cpp:2520-2558`, `Print::process`: make_brim populates m_brimMap and
  m_objectBrimAreas **before** _make_skirt; order is part of the boundary.
- `Print.cpp:2703-2778`, `Print::_make_skirt::append_skirt_loops_for_hull`:
  initial skirt flow/spacing, Extruder::e_per_mm conversion, per-call local
  filament budget, count extension until minimum is met and occupied reach.
  The budget counts full simplified skirt loop length, **not brim extrusion**.
- `Extruder.hpp:38-41`: filament flow ratio and cross-sectional-area conversion.
- `Print.cpp:2794-2819`: append each PrintObject's plate-frame brim-area outer
  contours to its occupied item. Skirt generation starts from that hull, not
  raw object bounds, brim-path centerlines or a twice-expanded outline.
- `Print.cpp:2830-2900`: fixed-point groups use configured nominal skirt reach
  (grouping_offset at 2794), not minimum-induced extra reach. Preserve source
  behavior even when extension changes export paths. PrintConfig.cpp:5751-5760
  explicitly documents final loops are not included in spacing validation.
- `Print.cpp:2898-2918`: combine_brims=false retains each object's brim with
  singleton owner ID; `2972-2996`: call local budget independently per skirt
  group, reverse loops, retain group/member/brim identity; flat m_skirt is only
  a preview/extents mirror.
- `GCode.cpp:4260-4299,4391-4452`: export actual materialized loop count with
  layer-height flow, seam and clipping; do not recompute minimum at export.
- `GCode.cpp:4456-4540`: generate_object_skirt_group once-owner group state,
  then generate_object_brim first-layer/owner tracking.
- `GCode.cpp:5325-5336`: selected normal pass emits group skirt then that
  object's independent brim then its content. A shared skirt must not emit
  all members' brims before the first member.
- `GCode.cpp:5748-5775`, extrude_loop: non-perimeter loops split at current
  nozzle position unless an explicit start point was supplied. Thus changing
  skirt extension can change subsequent brim segmentation/seam travel even
  with identical brim producer geometry; exact stream order matters.

Existing explicit-brim producer is the Brim.cpp:421-570,819-878 rewrite;
combined-brim connectivity at Print.cpp:2920-2970 remains deferred.

## Actual current Ares difference (measured, not a fabricated RED)

Evidence root: `${TMP_ROOT}/ares-skirt-min-brim-spec-oBAAYD/`.
Only external probe inputs/scripts/artifacts were created. Repository was
clean at exact base before/after Native runs, cargo build and public CLI probes.
Native official/seeded modes ran serially, CPU18, fresh per-case cache/output.
All **8/8** OS exits/result.json return_codes were 0 with actual positive XY
skirt **and brim** extrusion, not just option admission. Per-case input,
producer/wrapper hashes, command, full G-code/result/log, witness and affinity
samples remain; witness-summary.json and inputs/manifest.json are artifacts.
Actual sampled Native/thread affinity was 18, zero violations.

Current-main CLI was built with explicit RUSTUP_TOOLCHAIN=1.99.0,
CARGO_BUILD_JOBS=1, CARGO_INCREMENTAL=0, outer taskset CPU18; build exit0.
Owned target reuse is recorded (no other writer). Build head/status receipts
name the exact base; ares-binary.sha256 identifies the executed binary.
CLI probes record command, output/stderr/exit and actual sampled CLI mask18.
The two controls exit0 and emit real skirts and two brim blocks; the two
positive-minimum inputs exit1, stderr exactly:
`Error: UnsupportedProjectFeature("skirt_type per-object")`.
No positive-case Ares G-code exists. This is an observed public CLI difference,
not a committed API/CLI parity RED or a GREEN/full claim.

Each geometry pair retains the approved existing far-brim / near-brim 3MF
mesh, source identity, transforms, singleton instances and other options.
Only min_skirt_length changes 0 -> 20. Brim remains outer_only/width8/gap0,
combine_brims0; skirt loops2/distance6/height1; all other deferred features off.
Exact reserialized ZIP/JSON bytes are retained, not regenerated by future tests.

| Pair, both modes | Skirt groups / independent brim blocks | Positive skirt XY segments per group, 0 -> 20 | Relative skirt E mm per group, 0 -> 20 |
|---|---|---|---|
| far (X80/X140) | 2 / 2 | 106 -> 270, each group | 7.82392 -> 20.10565, each |
| near (X80/X110) | 1 / 2 | 103 -> 207 | 11.51098 -> 23.31296 |

Both modes have identical motion witnesses. Explicit M83/no M82 was checked;
E sums include only positive XY `; skirt` movement. Independent group targets
are visible in far; one shared target rather than one-per-member in near.
Brim extrusion is separate from skirt budget. Export clipping means emitted
E is not the general definition of the generation threshold.

Important observed coupling: far brim motion changes from 920 to 918 positive
XY segments per owner across the pair, while near is 919/922 in both. Far
summed brim E changes 39.06186 -> 39.06187 mm. An initial analysis assertion
that unchanged brim settings meant identical emitted brim motion **failed**;
this was analysis failure, not Native rejection. Raw streams remain untouched.
Do not assert equal brim bytes across min0->20 or normalize segmentation;
GCode's current-position splitting must be preserved by strict comparisons.
Zero-value control bytes must still remain unchanged across RED -> GREEN.

### Exact pure input hashes

| Case | SHA256 |
|---|---|
| far-0 | `418323ab5a32c53bf078be90b399086061f84fd5b6908e4c156633ce1e3476fc` |
| far-20 | `42515a9cfe67de97137dffaea78ad1088a0e2cb65fbd423c34e0756f7cdc91c9` |
| near-0 | `eda171d75ea65a378ce10aa5f3256347366e4bcf1adb73f142e91ecaae7eb9e8` |
| near-20 | `aaca766c8635170770996c20b9d287a3f5738fbc47034b4205e661fd820daf92` |

## Exact Rust destination / temporary compatibility shells

Under `crates/ares-core/src/project_slice/gcode_emit/`:

1. `skirt/generate.rs`: existing term
   `(print.min_skirt_length.0 != 0.0 && brim.iter().any(Option::is_some))`
   causes the measured rejection. Only narrow that term; independent brim
   predicate still defines eligible singleton/uncombined explicit outer brim.
2. `skirt/occupied.rs::items`, brim.rs::BrimPlan::covered_hull producer:
   consume area-derived hull in the correct plate frame. Existing mapping is
   a compatibility shell around m_objectBrimAreas, not a new obstacle model.
3. `skirt/loop_generation.rs::make_loops` / generate's per-group call:
   already port the source local budget. Preserve reset per group and include
   skirt loops only. Do not add independent brim length to satisfy minimum.
4. `skirt/member_groups.rs`, SkirtPlan ranges and layers/object_skirt.rs:
   reuse nominal fixed-point envelopes, group identity and once scheduling.
5. `layers/tool_entry.rs`, `layers/brim_emit.rs`, `brim/emit.rs` and existing
   motion loop emitter: retain skirt->owned brim->content sequencing and
   current-position split behavior. These scaffolds are temporary shells where
   they differ from cited GCode ownership/export, not permission to redesign
   unrelated motion. Admission removal alone is not assumed strict parity.

First raw difference determines any minimal source-cited repair within the
above interaction. Stop for producer identity/centering problems or a needed
wider boundary; request scope approval instead of touching another worker's
producer or relaxing controls.

## Included and deferred / post-approval gates

Included: far separate skirts and near shared skirt, positive minimum on
brim-aware occupied hulls, singleton independent outer brims, per-group budget,
actual full ordered export, unchanged paired zero controls.
Deferred: copied-instance brim footprint/once ownership, combine_brims,
auto/inner/ears/mixed brim policies, support/support-brim/raft, tower/draft,
sequential/multifilament ordering, nonuniform start angles, new placement rules.
Existing no-brim positive-minimum/copies behavior remains admitted unchanged.

After approval:
1. Independently commit these four pure-input fixtures and public API/CLI
   tests before production edits. Fresh two-mode Native-first witnesses must
   retain real extension and two owned brim blocks; expected baseline is
   2 zero controls PASS + 2 positive cases FAIL at retained skirt guard per
   mode. Review this RED separately; discovery CLI probes are not a substitute.
2. GREEN must require existing strict full ordered Native/API/CLI comparisons
   (only established generator metadata treatment), raw API/CLI equality,
   real skirt/brim motion and unchanged zero-control bytes. Never assert
   rejection as success or assume brim segmentation invariant across inputs.
3. Retain prior min-length8, brim4, grouping4, combined/single/zero-loop and
   relevant option-domain controls. No fixture/assertion deletion or weakening.
4. CPU18 serial modes, jobs1/threads2, explicit owned target policy, fresh
   Native cache/artifacts, actual descendant/thread affinity audits; exact
   submitted HEAD and clean status before/after each leg/gate. START/END inbox
   records; honor exclusive-full freezes.
5. Final-hash strict clippy/fmt/WASM; core remains platform-neutral, no I/O/UI.
   Canonical full/workspace need separately scheduled coordinator authorization.
   Coordinator alone merges/pushes. Submit only SPEC now, no implementation.
