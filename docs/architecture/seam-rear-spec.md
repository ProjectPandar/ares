# SPEC: SeamPlacer spRear candidate comparison and alignment

Status: **SPEC-REVIEW-REQUEST, docs only**. Proposal approved by coordinator.
Base approved physical-plane GREEN
`4dbba4fc09b4b84033797ab83f2941e5d8484b95`, not a new branch off stack24.
No RED or production edit before exact SPEC approval. [ARD](ard-seam-rear.md).

## Upstream rewrite boundary and destination

Pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`; the following source is
`OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp`.

| Upstream function/data | Included behavior | Exact Rust destination |
| --- | --- | --- |
| `SeamComparator::setup`, `is_first_better:753-795`, especially777-779 | For spRear, larger candidateY wins after existing type/overhang/embedding priorities; equalY uses the existing visibility/angle comparison. | `project_slice/seam_placement/alignment.rs::{LayerPlan,is_better,is_better_between}`; retain object-specific placement identity through existing preparation. |
| `is_first_not_much_worse:802-849`, especially839-841 | After earlier priorities, spRear uses `a.y + seam_align_score_tolerance * 5.0f > b.y`; preserve strict comparison and original constants. | `alignment.rs::is_not_much_worse`. |
| `pick_seam_point:917-928` | Select the best candidate with the mode-specific comparator; not an independent polygon rotation. | `alignment/preparation.rs::{prepare,best_candidate}`. |
| `align_seam_points:1237-1425`, especially1364-1366 | Multiply the existing accumulated seam-string length by0.3 for rear before choosing spline segment count; reuse existing observations, weights, projection and fit. | `alignment.rs::align`; existing spline implementation unchanged. |
| `SeamPlacer::init:1427-1492`, especially1486-1492 | spRear enters candidate selection and seam alignment alongside aligned/aligned_back. | `seam_placement/runtime.rs::placement_modes`, `seam_placement.rs::apply_objects`. |

Rust paths are relative to `crates/ares-core/src/`. Configuration already accepts
`ProcessSeamPosition::Back` for the public `seam_position="back"` value. The
current project runtime sends Back to `_ => None`; consequently it never reaches
these project candidate/comparison/alignment consumers. Replace that temporary
Ares bypass with the cited upstream specialization. Do not introduce a new
pipeline, a second candidate producer or an unrelated scoring abstraction.

`perimeters/seams.rs::first_max_y_index` is an existing basic-perimeter API
helper, not a port of this project `SeamPlacer` boundary. Leave its independent
API untouched; do not import/re-export it, rotate loops directly, or keep it as
a fallback for this slice. Rear is NOT simply "pick the first max-Y vertex":
priority ordering, equal-Y behavior, similarity and fitted seam positions matter.

## Approved discovery witness (not yet formal RED on this base)

Discovery was run at exact stack24
`4351a644d6683555ae48e44ea7f898a66f75d585`, with a freshly built dev/debug bare
CLI and fresh official2.4.2 Native. It is not relabeled as an execution on4dbb,
a public validated-API RED, or seeded evidence. This SPEC's base change is the
coordinator's explicit instruction to depend on the approved physical-plane
GREEN, avoiding overlapping stack25 edits. That GREEN changes only plane
handoff/helper deletion; this single-region witness has no missing region slots.
Formal RED must verify the premise at its actual frozen hash on4dbb, not assume
it or silently substitute inputs if it changes.

Minimal geometry: one closed10x10x0.8mm triangular prism,6 vertices,8 in-bounds
triangles, one object/part/filament, four0.2mm layers, Classic two walls; support,
brim and prime tower off. Finite ordinary geometry, not UB/crash behavior.

- Aligned control SHA:
  `8cda6c553c73d4f55ab638781b1e42c1edc9291652b73ca66c1c529abc2ac6ba`.
- Back positive SHA:
  `d4bfafd0fff8e1f55df0310775f17f9af600c3e1863049e580c660838fd31afa`.
- Only ZIP member `Metadata/project_settings.config` differs; only decoded
  setting `seam_position` differs, aligned→back. Every mesh/relationship/
  transform/other setting is identical. Whole-input and ZIP-entry SHA manifest
  plus singleton-delta proof are retained outside git until RED is authorized.

Both Native invocations rc0; both bare Ares CLI invocations rc0, with complete
configuration-footer streams. Aligned control whole comparison PASS using only
the existing generator/vendor timestamp normalization. Positive first complete
strict difference is normalized byte451, line18, column9:

| Native | Ares |
| --- | --- |
| `M73 P0 R1` | `M73 P0 R2` |

Lengths33774 versus34040 normalized bytes. The first differing movement is
motion9 (diagnostic G0/G1/G2/G3 projection ONLY), raw line64 in both streams:

| Native | Ares |
| --- | --- |
| `G1 X110 Y113.446 ; move to first perimeter point` | `G1 X110.43 Y112.509 ; move to first perimeter point` |

Native starts the first inner perimeter at the rear apex; Ares starts one
corner/segment away despite accepting and echoing back. Native's next extrusion
is toward109.57,112.509, while Ares first extrudes to110,113.446. This is an
observable seam start difference, not evidence for a new clock/M73 owner.
Complete comparison must continue to retain every M73/motion/arc/statistics/
configuration/footer byte. No timing fix or filtering is authorized.

History proof freshly fetched origin/main41f2 and ci/stack-244351. Scoped
`git log -G 'ProcessSeamPosition::Back|SeamPosition::Back|spRear'` over project
seam runtime/alignment has zero hits on main,4351 and queued4dbb. Broader Back
search hits `cf31ec047`, the distinct aligned_back front-visibility bias.
Legacy basic-perimeter Back belongs to squash5977b66b6 and is explicitly
acknowledged, not mistaken for a project spRear implementation.

## Included and deferred behavior

Included: object-specific Back routing through existing candidate preparation;
source rear comparison priorities/equal-Y handling; source rear similarity and
string-length fit specialization; existing loop association/projection/emission
consumes the resulting choice without independent redesign. Borrow physical
slice_z exactly as approved4dbb; preserve its producer and conversion.

Deferred/unchanged: painted enforcers/blockers and their candidate priority
implementation, scarf, new random/nearest behavior, aligned_back visibility
bias, overhang/embedding production, mesh sampling/BVH/KD-tree algorithms,
angle/visibility coefficients, plane/layer/region planning, perimeter generation
or order, loop geometry, travel, clipping/arcs, M73/caches/clocks and libvgcode.
Existing higher-priority fields remain ordered as upstream; no invented paint
state, guard, validation, coefficient or heuristic is introduced to simulate them.
Shared code must preserve existing aligned/aligned_back/random/nearest semantics.

No schema/namespace relaxation or unsupported-feature guard retirement: the
witness is already admitted and produces complete output in both engines.
Earlier external paint_seam exploration failed Ares XML admission and is
excluded, not a secondary proposal, fixture source or reason to bypass checks.

## SPEC → RED → GREEN gates

1. **Exact-hash SPEC REVIEW-REQUEST**, docs only, direct base4dbb. Await approval
   before adding permanent tests/fixtures or touching production. Preserve the
   approved physical-plane chain; do not rebase it or merge without coordinator.
2. After SPEC approval, freeze one paired public RED and the exact two inputs /
   ZIP-entry manifest, no Native products/probes in git. Per mode (official and
   seeded): fresh Native aligned control→public validated byte API with default
   overrides+real bare CLI→success receipts/raw API==CLI/BOTH complete strict
   PASS; then fresh Native back positive→both public calls and receipts BEFORE
   success/raw/target assertions. Persist BOTH full comparison results before
   assertions. Positive asserts equality and must genuinely FAIL until GREEN,
   not pass on expected errors, an environment flag or a Native-only observation.
3. Native first-motion witness is mandatory: retain complete positive Native
   stream and exact line64 first-perimeter-point block; retain corresponding
   public streams. Offline first-motion projection is localization only, not
   a comparator. Missing streams are NOT EVALUATED, not a qualified RED. If
   base/profile changes the aligned control or removes the back premise, STOP
   and report; no input replacement/control search to manufacture a RED.
4. Proposed frozen-witness budget: four Native commands (control+positive in
   official+seeded), CPU15/jobs1/nextest threads1, fresh per-mode cache/artifacts/
   TMPDIR, load<60 before builds/every Native, START/END and complete authoritative
   snapshots. No Native request retry-to-success. Predict the known complete
   positive failure and first-motion gap; record actual offsets rather than
   assuming dev/debug discovery offsets across build profiles.
5. **Exact RED REVIEW-REQUEST**, await approval/separate GREEN authorization.
   GREEN ports ONLY the source branches/destinations above, reusing candidates,
   planes and spline; no max-Y helper/shortcut, fallback, feature flag, new
   nearest/random/paint path or M73 changes. Freeze RED unchanged. Residual whole
   mismatch after this bounded implementation means STOP/report, not widening.
6. **Exact GREEN REVIEW-REQUEST** with both-mode frozen control+positive whole
   PASS/raw API==CLI and Native first-motion parity; existing seam/nearest/
   aligned/rear regressions on CPU15, fmt, strict scoped core+CLI test clippy,
   core+adapter WASM checks, changed Rust files≤400LOC. Audit optional Native
   gates (notably ARES_PARITY_CLUSTER), verifying actual outcomes rather than
   nominal test exits. Preserve browser WASM/Windows/macOS/Linux API neutrality.
   Coordinator owns full/workspace scheduling/integration. RED merges only with
   its GREEN; no worker full/workspace/merge/push authorization is implied.

## Durable discovery

`${LOCAL_EVIDENCE_ROOT}/worker4/rear-seam-slice-proposal.md`;
root `rear-seam-proposal-4351-vqtu`, final archive SHA
`0e65748d97f19c688bea5012d77de117f1fe26383103f6a26b55c8a950552fc9`.
Includes complete Native/CLI streams, rc/stdout/stderr, SHA/ZIP and delta proofs,
first-motion diagnostic, settings/footer proof, base/queued runtime snapshots,
fresh git history, source/binary hashes and4351 build log. No public API/seeded
execution or production GREEN is claimed for this new slice yet.
