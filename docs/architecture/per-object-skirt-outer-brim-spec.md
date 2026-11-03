# SPEC: explicit outer brim in multi-object per-object skirt groups

Status: **SPEC-REVIEW-REQUEST — no production or test code authorized**.
Base: `73e8e712e021d88ed136ba92f76bfd017477687a`.
Branch: `worker1/per-object-skirt-brim-spec`.
This is a bounded `libslic3r` rewrite slice, not an Ares pipeline feature.

## Non-negotiable decisions

- Extend only the brim portion deferred by the merged per-object skirt slice.
  Keep PrintObject membership, fixed-point envelope merging, plate-frame
  occupied geometry, and once-before-first-member skirt emission.
- Include explicit `outer_only`, positive width, `combine_brims=0`, one
  printable instance per PrintObject, simple disjoint solid first-layer islands,
  one filament, by-layer order, uniform skirt angle, positive skirt loops,
  `min_skirt_length=0`. Existing no-brim controls stay unchanged.
- Support, support-brim, raft, tower, draft, sequential, multiple filaments,
  combined brims, auto/inner/ears brims and repeated-instance brim ownership
  remain deferred. No global combined-skirt fallback or new Unsupported error.
- Preserve the original existing guard predicate `resolved.objects.len()>1`.
  Only narrow its brim term for the admitted source-owned subset; do not
  broaden other rejection terms or introduce general option policy.
- No code before approval. After approval: Native-first public RED before
  GREEN; no assertion weakening, classifier/comparator changes, skipped cases,
  or Native-output commits. Files remain <=400 LOC; split mechanically first
  if necessary. Merge/push is coordinator-only.

## Owning upstream boundary

Vendored `OrcaSlicer/src/libslic3r`:

1. `Print.cpp:2520-2558`, brim preparation in `Print::process`: collect unique
   PrintObject IDs, call `make_brim` with `m_brimMap`, `m_supportBrimMap`,
   `m_objectBrimAreas`, `m_supportBrimAreas`; `_make_skirt` runs **after** brim.
2. `Print.cpp:2794-2819`, `Print::_make_skirt::SkirtGroupItem`: translate all
   object instances, then append outer contours of the object's **plate-frame
   brim areas** to the object's occupied item. Do not substitute brim path
   bounds, raw object bounds, or a second additive skirt-distance expansion.
3. `Print.cpp:2830-2900`: rebuild expanded group envelopes until merging
   stabilizes; brim occupancy can merge otherwise-disjoint skirt groups.
4. `Print.cpp:2898-2918`, `make_group_brims`: for `combine_brims=false`, retain
   each object's brim collection and singleton owner ID. `2972-2996` installs
   group skirt, group object IDs and owned brims in `SkirtBrimGroup`.
5. `GCode.cpp:4456-4484`, group lookup and `generate_object_skirt_group`;
   `4486-4540`, `generate_object_brim`: first layer only, matching owner,
   `m_objsWithBrim` once-emission tracking, plate origin.
6. `GCode.cpp:5325-5336`: selected normal extrusion pass emits group skirt,
   then that object's brim, then its ordinary content. Sharing a skirt does
   **not** emit the second object's independent brim before the first object.
7. Existing brim area/loop producer: `Brim.cpp:421-570,819-878`, explicit
   `outer_inner_brim_area` / `makeBrimInfillImpl`. This proposal consumes that
   boundary, not an independently designed brim algorithm.

Combined-brim contact union, cleanup and shared ownership at
`Print.cpp:2920-2970` are explicitly outside this slice.

## Exact Rust destination and current gap

Under `crates/ares-core/src/project_slice/gcode_emit/`:

- `skirt/generate.rs`: currently rejects positive per-object skirts with more
  than one resolved source when any `BrimPlan` exists. Narrow only that brim
  admission term for the included subset; remaining terms are retained.
- `skirt/occupied.rs::items`: already appends `BrimPlan::covered_hull`, shifted
  by `print_object_origin`, to the PrintObject item. Audit this against upstream
  plate-frame `m_objectBrimAreas`; do not shift twice or include extrusion
  centerline bounds instead of brim-area contour occupancy.
- `skirt/member_groups.rs`, `loop_generation.rs`, `layers/object_skirt.rs`:
  reuse merged source-owned membership/fixed-point/emission machinery, not
  another grouping algorithm. Flat skirt loops remain preview/extents only.
- `brim.rs::BrimPlan::generate`, `brim/emit.rs`: existing explicit-brim area
  and path producer remains the producer. Its local area-derived convex hull
  can represent contour occupancy for skirt convex-hull purposes; validate
  quantization/offset against Native before relying on that equivalence.
- `layers/brim_emit.rs`, `layers/tool_entry.rs`: current emitter is a per-entry
  compatibility shell; consume PrintObject ownership on the correct normal
  pass, first layer, after the group skirt, before object content. For this
  one-instance/uncombined slice, singleton brim owner equals PrintObject
  identity. Do not invent shared brim owners or mark another member done.

The existing scaffolding is only a compatibility shell wherever its order,
coordinate frame or once-emission semantics differ from these upstream
functions. First raw mismatch determines the minimal repair; do not assume
that deleting the admission term alone gives parity. Producer identity or
centering defects are reported to their owner, not repaired here.

## Native admission and actual-effect proof (completed; not Ares RED/GREEN)

Evidence: `${TMP_ROOT}/ares-skirt-brim-native-ANfbTd/`.
`head.txt` is the exact base above; `status-before.txt` and `status-after.txt`
are empty. No cargo/nextest/nix build or Ares execution occurred.
Official and seeded Native ran serially, outer CPU18; each case has fresh
cache/artifact directories, exact input, command, wrapper/binary/input hashes,
Native `result.json`, complete G-code, stdout/stderr log and affinity samples.
All 8 return_code/OS exits were 0. Sampled Native descendant/thread masks were
18 with zero violations. `witness-summary.json` and per-case `witness.json`
retain real positive-E XY skirt **and brim** motion, not just config legality.

Inputs derive from the retained disjoint fixture (source SHA
`c98041c6033ffecbd0dace6feb89af8f6f0c286650f9241bc52cd617cbbb3317`).
Only instance X (far 80/140, near 80/110), `brim_type`, `brim_width` and explicit
`combine_brims=0` change; no mesh, object identity or independent-mesh variant.
Objects are distinct PrintObjects with identical shared cube mesh and a single
instance each. Brim width 8, gap 0; skirt loops 2, distance 6, height 1.

| Input | SHA256 | Skirt TYPE groups / brim TYPE blocks, each mode |
|---|---|---|
| far-control | `903a9f44ba478b4aad89b917a92194d2b662eba29bdefa60c791da3bf5556708` | 2 / 0 |
| far-brim | `43c012f6a5d1e566834cbb9ada7763a8395dcf270b2b0bb923750e53a8af8ca8` | 2 / 2 |
| near-control | `1cb8cf433de76d0dfa8f70b9a013690c39ecae5b6396689099b476b974342809` | 2 / 0 |
| near-brim | `8e28413bbff6addfc83e410ebc7737e1eec3efdba2e3ed3a94ea47a5f2fa752b` | 1 / 2 |

Both modes have identical witness counts/bounds. Near-control skirts span
X 68.434..121.566; near-brim skirt spans X 60.893..129.107 and Y
90.904..129.107. Near-brim contains 103 positive XY skirt segments and 1841
positive XY brim segments. Far-brim contains 212 and 1840 respectively.
Near geometry is unchanged across its pair: **brim occupancy alone changes
2 independent skirt emissions into 1 shared group, while retaining 2
independent brim emissions**. Far brim remains 2 groups. This proves admission,
actual brim/skirt effect and the intended merge path; it is not a full parity
claim. Brim TYPE blocks are emission blocks, not asserted loop counts.

## Post-approval RED and GREEN gates

1. Commit the exact four witness inputs/hashes and a public parity test module
   before production edits. No committed Native outputs. Record paired real
   motion witnesses, role ordering and input hashes as repeatable artifacts.
2. Fresh official/seeded CPU18 Native runs first. For far/near brim inputs,
   current public API and CLI must fail at the retained
   `UnsupportedProjectFeature("skirt_type per-object")`; controls must pass.
   That rejection is source-predicted, **not yet executed or claimed RED**.
   Stop on dependency/environment failures; retain raw records.
3. GREEN requires Native strict full ordered G-code comparisons (only existing
   generator metadata treatment), API/CLI raw equality, skirt groups 2/1,
   nonzero brim motion, independent first-layer brim ownership/order, and
   unchanged paired controls. No geometry normalization or admission-only gate.
4. Retain prior four per-object grouping fixtures, existing combined/zero-loop
   controls, single-object skirt_type domain, and existing explicit-brim
   controls unchanged. Report first raw differences before any wider repair.
5. CPU18 directed serial modes, fresh Native caches/artifacts, explicit target
   policy and actual descendant/thread affinity audit. Log exact submitted
   HEAD and clean status before/after each leg; no stale hash labels.
6. Strict clippy, fmt, WASM checks on the final hash. Canonical full and
   workspace nextest require separately allocated coordinator windows; do not
   start them with SPEC approval alone. START/END and exact evidence paths go
   to the inbox. Stop for any exclusive-full freeze.

## Deferred / stop conditions

No support/raft occupied-hull or height changes, no obstacle-only tower group,
no draft/min-filament/sequential/multifilament scope expansion. Copies with
brim need the upstream all-instance plate-area producer and once-owner proof
and remain deferred; do not copy the first instance's brim footprint to stand
in for that producer. No automatic brim decisions, mixed brim policy or combined
brim connectivity rewrite. If the bounded subset cannot be admitted by only
narrowing the existing brim term, request a scope decision rather than add a
new guard/error. No UI/filesystem logic enters core; WASM remains a required
platform boundary. Only this SPEC is submitted for approval.
