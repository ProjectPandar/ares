# SPEC: modifier-volume per-feature filament identifiers

Status: **SPEC APPROVED a2ffbb77bcf205b657e70f17245b38c9b78cd2d2; public RED authorized with per-role witnesses.**
The RED extension below records control preflight and all-six Native qualification.
No guard or production change in the RED; GREEN needs separate approval.
Base `d84ad64aea77e21610f0a780d729b6ca0c5c3f95` (approved triangle chain,
queued for coordinator merge). The original SPEC was docs-only. This RED adds
six independent public tests and input-only 3MF/SHA fixtures, no production edit. No full/workspace run. This ports an upstream region-config slice,
not an Ares-owned slicing pipeline. [ARD](ard-modifier-feature-filament.md).

## 1. Source owner, precedence, and destination

Orca pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `src/libslic3r/`:

- `PrintObject.cpp::FeatureFilamentOverrideMask:3592-3600` and
  `apply_to_print_region_config:3602-3662`: copy non-extruder fields first;
  positive feature filament IDs set the feature value and explicit bit. A zero
  clears that bit (it does not immediately overwrite the inherited value).
  Afterwards a **positive base `extruder`** applies only to features whose bits
  are clear. This is the owner of modifier base-extruder precedence.
- `region_config_from_model_volume:3664-3705`: start from print defaults for a
  model part, but from the already-resolved parent region for a modifier.
  Model parts seed explicit bits from defaults and apply object config;
  modifiers start with clear bits and do not reapply object config. Apply volume,
  then material config, then model-part-only layer config; finally resolve/clamp
  feature IDs. `extruder=0`/absent supplies no new fallback.
- Thus modifier `sparse_infill_filament_id=2, extruder=1` keeps sparse on2,
  while unprotected features fall back to1, even if the parent had explicit
  feature IDs. Modifier `sparse=0, extruder=1` clears protection and falls back
  to1; modifier `sparse=0, extruder=0` leaves inherited sparse until final
  normalization. These are source contracts, **not newly probed cases**.
- `PrintApply.cpp:1032-1055` creates modifier regions from intersecting parent
  regions using that resolver, retaining parent relationships. The cited
  `:810-830` and `:1060-1075` are **painting** override/update paths: after
  base/modifier resolution they set all six feature IDs to `extruder_id` /
  `painted_extruder_id`. Do not confuse that final painting order with modifier
  `extruder` fallback; painting implementation is not this slice.
- `PrintRegion.cpp::extruder:6-23`, especially `:13-15`, selects sparse infill's
  1-based filament ID for `frInfill`; `collect_object_printing_extruders:73-94`,
  especially `:81-86`, collects a 0-based selected tool when sparse density>0.
  Logical ID2 therefore yields T1; mere T1 occurrences in machine templates are
  not proof of modifier-region extrusion.

Rust boundary already contains much of this rewrite:

| Destination | Current behavior / bounded future role |
|---|---|
| `project/effective_config/candidates.rs::reject_unsupported_object_sources` / `first_unsupported_modifier_key:147-176` | Rejects any ParameterModifier carrying sparse/internal-solid/top/bottom/outer/inner feature filament field, even before region resolution. Positive statically predicts `UnsupportedProjectFeature("sparse_infill_filament_id")`; no public call has been made yet. |
| `project/region_graph.rs::add_modifier:137-191` | Intersects parent bounds, invokes `RegionOptions::resolve(RegionBase::Modifier { parent }, volume overrides)`, registers distinct child config/parent link. |
| `options/region_options/merge.rs::resolve`, `FeatureFilamentOverrideMask`, `apply_overrides`, `apply_feature` | Already expresses clear modifier mask, inherited parent, feature-before-base-extruder fallback and normalization. Reuse the source-cited resolver, do not invent a modifier-only material engine. |

The prospective production boundary is **only this six-key modifier feature
filament admission gate**, after a qualifying public RED and separate GREEN
approval. No other guards may be touched. **Owner condition: only branches with
their own public RED witness may retire.** All six have now qualified in Native
below, but each still needs its own public RED; no blanket guard retirement.
No resolver rewrite is currently justified. Existing scaffolding remains only a compatibility shell
around upstream region config, not authority to design different precedence.

## 2. Completed Native discovery (before SPEC)

Durable root `/home/indexyz/.ares-evidence/worker4/modifier-feature-native-bBjU`.
All inputs frozen externally before any Native invocation, with full ZIP-entry
hashes and deterministic derivation. They are slicing inputs, not stored oracle
outputs. **No public API/CLI call and no production/test edit in discovery.**

Geometry/accepted modifier source:
`tests/parity/modifier-sparse-loss/A-C1.3mf`, SHA
`7fd8549eacaff99b71fe3b5eaea58d365a545d831174255cdc2bec1a69005619`.
Two-filament complete settings donor:
`tests/parity/validation/multi-filament-bridge-anchor-narrowed.3mf`, SHA
`22d3813ddbbfbc40bd3955fb35d5f6d19b7d8ee23472e7aa2e354e50a23cfe82`.

Common derivation: retain all A-C1 geometry/transforms/resource parts and its
modifier sparse pattern/density/width. Replace only project_settings.config
with the donor's complete two-filament settings (prime tower/support disabled,
global sparse ID0, object extruder1). Add matching plate `filament_maps="1 1"`
to both inputs. Modifier `extruder` is absent, hence no modifier base fallback.

- Control SHA `68e5b1010eb40d8ec44b55b349eea04d6088ea6b49212e51402a6f2ea171b652`:
  modifier has **no feature filament key**.
- Positive SHA `3b81f66c1e1df0b5fd39f9de8e426b2e1d24376ad5d23110a4a5b607d08c00f2`:
  add only `<metadata key="sparse_infill_filament_id" value="2"/>` inside
  modifier part3. Pair differs in model_settings.config only; settings/geometry
  identical. No arrangement, hidden loaded JSONs or per-case overrides.

CPU15, official + seeded canonical wrappers, bare `--slice 0 --outputdir`, fresh
case/mode cache/TMPDIR/directories, load<60 before each Native, START/END receipts.
Exactly four Native slices; all exit0, full CONFIG_BLOCK_END footer, no timeout,
signal or observed rejection. No case/input retry or fallback.

| Observation | official | seeded |
|---|---|---|
| Control sparse positive-E XY moves | 2509 T0, zero T1 | same |
| Positive sparse moves | 1714 T0 + 780 T1 | same |
| Positive T1 sparse segments inside modifier bbox | all780 | all780 |
| Output positive vs control, generator-only normalized | different | different |

Control normalized SHA
`dd5e717540b4d0ae2e530090e3a8329d5de9b21f1635bf2e75620240fa771be5`;
positive `d2cfcd9e9fc09bdfdad10678c6dd02b5828e5165fe58c6a728338243800eee85`
in both modes. This is discovery evidence, not public parity PASS.

Spatial proof: modifier world bbox X[133,138], Y[118,138], Z[0,10] (mesh
[-5,5], component0.5x/2y scale, center125.5/138/5, root item+10/-10).
Track actual T command, M82/M83/E deltas and emitted `; FEATURE: Sparse infill`.
Positive line1880 under T1: `(134.506,118.993)` -> `(136.135,118.993)`,
`G1 X136.135 Y118.993 E.05401`; line1881 -> `(133.204,124.07) E.19447`.
Both segment endpoints are inside the modifier; control has no such T1 sparse
move. This excludes start/end/purge template T1 as the sole evidence.

Observer correction is retained transparently: the initial parser accepted only
`;TYPE:` and produced zero observations despite actual T1 commands. It paused
after the two official legs. Pure offline re-observation added the donor's
`; FEATURE:` dialect, on **unchanged existing G-code**, then qualified the local
proof before running the still-unexecuted seeded legs once. Initial script/log
and observations are retained. No Native retry-to-success or input substitution.

## 3. Approved extension: control first, then six role-qualified RED inputs

Before adding any role to the public RED, fresh Native/public control was run
in both modes at a2ff: API success, real CLI exit0, raw API==CLI, whole strict
Native/API/CLI PASS through the unchanged golden.rs comparator. Evidence root:
`/home/indexyz/.ares-evidence/worker4/modifier-control-public-okRS`.
No input or settings substitution was needed.

Then the other five single-key positives were frozen from the exact control.
Each adds only that role's filament_id=2 inside modifier part3; the same sparse
input retains its original SHA. Native qualification (both modes, CPU15) used
actual FEATURE role, T1 selection, positive E and both segment endpoints inside
X[133,138]/Y[118,138], plus output divergence vs the control. Every new leg exit0
and complete footer; no input/case retry, no nonzero modifier extruder case.
Evidence root `/home/indexyz/.ares-evidence/worker4/modifier-six-role-native-E9JO`.

| Feature key prefix | Actual FEATURE comment | Local T1 segments (each mode) | Eligible for public RED |
|---|---|---|---|
| sparse_infill | Sparse infill | 780 | yes |
| internal_solid | Internal solid infill | 405 | yes |
| top_surface | Top surface | 43 | yes |
| bottom_surface | Bottom surface | 63 | yes |
| outer_wall | Outer wall | 200 | yes |
| inner_wall | Inner wall | 196 | yes |

All corresponding control local T1 counts are zero. No unqualified role in this
frozen matrix; any role losing qualification must keep its guard. The committed
`tests/parity/modifier-feature-filament/manifest.json` pins control + six
positive 3MF SHA/entry identities, with no Native G-code/expected output.

Six independent `modifier_feature_filament_*_matches_native` tests share a
bounded helper: from_env/reference gate; fresh runtime Native control, whole
public control first; fresh positive Native and local T1 qualification; then
execute positive API+real CLI before target success assertions. Each role has
its own artifact/error receipt and own failing assertion rather than a loop
that stops at the first role. Per-role targeted runs stop on a control failure;
no test/comparator/guard fallback. No GREEN branch may retire until its own
public RED is reviewed. Native all-role admission/effect is not public parity.

## 4. Public RED plan and gate (original contract, retained)

First review this source boundary/discovery; do not treat successful Native-only
discovery as a public RED. Freeze the exact pair/manifest in git only for the
separately reviewed test. Every enabled run must generate fresh Native control/
positive (and an approved repeat if required), then execute validated public
`slice_project_plate_validated` and real bare CLI on identical bare bytes.
Use established from_env/reference gate; no Native-only pass, historical cache
oracle, external reference injection or export/load-only substitute.

Public control first must succeed in both adapters, raw API==CLI, **complete**
strict generator-only Native/API/CLI equality. No claim that this control
already passes: it has not been public-tested. Any control rejection or output
delta disqualifies the RED; STOP/report without repairing other guards/M73.

Execute positive API and CLI before target assertions, preserve both errors,
then assert success, raw equality and the same whole strict comparisons. Current
guard is predicted to reject sparse_infill_filament_id; the test must genuinely
FAIL its target-success assertion, never PASS an is_err/expected-error/assert_ne
branch. Missing stream means comparison NOT EVALUATED, never missing-output
equality. T1/locality assertions supplement, not replace whole-stream parity.
Native positive/control divergence is discovery qualification, not a weakened
Native-vs-Ares comparator. Retain M73/config/motion/arcs/statistics/footer.

Separate GREEN authorization after RED review. Frozen test/inputs/comparator
stay unchanged; both modes must become whole strict PASS with raw API==CLI and
unchanged control. Targeted modifier/3MF/load regression plus fmt/strict clippy/
WASM remain the exit gates; coordinator alone schedules full/workspace/merge.

## 5. Included / deferred and failure inventory

Included source contract: modifier inherits parent region, its explicit positive
feature ID beats same-scope base extruder fallback, zero/absent semantics and
final normalization remain as the existing upstream/Rust resolver. Initial
runtime proof/public RED initially concerned sparse only; the approved extension
now supplies six separately qualified role cases, all with modifier extruder
absent. Nonzero modifier extruder remains unprobed and is not added here.
No new six-role engine, new material system, tool scheduler, painting engine or
CLI interface is designed. Other admission guards remain intact.

Deferred runtime matrices: modifier extruder!=0,
zero/out-of-range IDs, material/layer overrides, later/overlapping modifiers,
painted regions, more filaments/maps/printers, prime tower/flush changes. Their
source precedence is recorded above, not claimed empirically proven here.
No unrelated M73/config/geometry repairs to obtain a qualifying control.

| Failure | Decision |
|---|---|
| Native rejects/signals/times out or no positive-control divergence | STOP; no SPEC/RED promotion from that pair and no retry/input substitution. |
| No actual positive-E T1 sparse segments in modifier region | STOP qualification; raw T1 tokens alone are insufficient. Distinguish observer defect from Native behavior using saved outputs only. |
| Pair changes non-target metadata/config/geometry | STOP: redo review, not runtime tweaking. |
| Public control not whole strict PASS/raw-identical | RED invalid; report first difference, do not alter other guards/comparator. |
| Positive same-input public equality already holds | NO-RED STOP; do not force a failure. |
| Positive public rejection with complete Native/control | Target-success test FAIL; preserve both API/CLI receipts. |
| Missing public stream | NOT EVALUATED for raw/whole comparison. |
| GREEN leaves scheduling/role/motion/footer difference | STOP; no downstream repair or feature-family expansion. |
| Nonzero modifier extruder case contradicts inherited-mask precedence | Requires separate bounded evidence; do not invent an Ares-specific precedence. |
