# SPEC RR: consumed GUI painted fuzzy-skin rewrite

Status: bounded discovery confirms admission rejection; request independent SPEC
approval before public RED. No production edits, RED or GREEN authorized here.
Discovery/base:92ce9b02c9f3e1e332347350fa199128209ba974 (frozen accepted beam tip).
Upstream:8500fcdccaa10b5099ac20d252af3a7c560046f1.
[Non-negotiable ARD](ard-painted-fuzzy-skin.md).

## Owning source boundary, not a new Ares pipeline

All C++ paths below are under OrcaSlicer/src/; Rust paths are relative to
crates/ares-core/src/.

| Upstream owner | Rust destination / existing incomplete shell |
|---|---|
| libslic3r/Format/bbs_3mf.cpp330,3824,5058–5078,7339–7346: canonical paint_fuzzy_skin ↔ ModelVolume::fuzzy_skin_facets | project/xml/model.rs, project/model_xml.rs::Triangle, project/load/mesh_prepare.rs, project/domain/mesh.rs and existing project/facets_annotation data/selector codec |
| libslic3r/PrintApply.cpp1081–1110: derive actual FuzzySkinPaintedRegion configs/parent IDs and sort by parent region ID | existing project/region_graph and project_slice/volume_regions shared real registry; no independent registry |
| libslic3r/MultiMaterialSegmentation.cpp2211–2226 fuzzy wrapper →1955–2194 segmentation_by_painting | private project_slice/region_slices/fuzzy_painted with source-equivalent reuse of existing painted projection/graph/cut machinery; current MMU-only projection is not fuzzy support |
| libslic3r/PrintObjectSlice.cpp1033–1141 apply_fuzzy_skin_segmentation;1227–1241 caller | private region-slice apply and load_stages post-MMU/pre-interlocking handoff, replacing the absent operation at this source position |
| libslic3r/Feature/FuzzySkin/FuzzySkin.cpp415–473 compatible fuzzy region configs,502–559 merging,561–684 classic masked apply | existing private project_slice/perimeters/classic/fuzzy_skin.rs, traversal/types.rs and materialize/path.rs seams with real compatible-region masks, not a second perimeter engine |

Current staged print_apply/generate_fuzzy_*_state modules only carry config
markers/registry concepts. They are temporary upstream compatibility shells, not
an actual project graph/segmentation consumer. Existing classic/global fuzzy
support carries a single FuzzySkinConfig; admitting the XML then using that
whole-polygon helper without real masks is not a complete rewrite of this slice.

## Discovery: actual reject, not silent ignore

Evidence: /home/indexyz/ares-evidence/worker1/painted-fuzzy-discovery-92ce9b02c/.
Two stock inputs, all geometry/instances/preset container preserved from
legacy-brim-ears/O0.3mf (SHA72bc653d1eee964032dc2a8a33f7c6cd070685e9a8385e9f3d083584281673ec).
The stock one-filament cube has8vertices/12triangles. Both inputs change only
fuzzy_skin=disabled_fuzzy→none in project settings; target additionally gives
original triangles10/11 on the complete local+X vertical face the canonical
unprefixed paint_fuzzy_skin="4" (unsplit leaf state1). No replacement geometry.

| Case | SHA256 |
|---|---|
| stock-fuzzy-unpainted |603c3228513e194c6fccabea843ba3640b3b96aefa7b4728ea902c9f59fa8357|
| stock-fuzzy-painted-face |3d3401362eb9904e84999b2e2ab6368137e521702f4ebb26dec779608ada0f22|

GUI GLGizmoFuzzySkin.cpp326–354 explicitly enables painted fuzzy by setting None,
not All;367–410 updates/loads fuzzy_skin_facets. TriangleSelector.cpp1692–1752,
Model.cpp3545–3595 and the bbs_3mf writer prove the annotation encoding. This is
GUI-serializer-equivalent source construction on the stock project, not a claim
of a live desktop GUI-save session or runtime platform qualification.

Kept stock classic/Ripple, thickness0.2, point distance0.3, first-layer0,
displacement,15ripples,50%offset,offset-layer spacing1; support/tower disabled.
Ripple89–156/296–300 avoids stochastic random_value24–31 and point placement.
8 actual Native processes (2inputs×2modes×reference/replay) exit0, settings readback
and positive model XY+extrusion; painting changes ordered XYE in both modes.
Every complete reference/replay stream is byte-stable after ONLY the established
generator-line allowance. Raw file equality is1/4 pairs; other3 differ only in
that timestamp line. No stochastic waiver/tolerance/filtering/retry. If review
requires literal timestamp-inclusive raw equality, these3 pairs are not raw-byte
stable; do not silently qualify them under that stronger contract.

All8 Native processes complete before2 real CLI calls. Unpainted succeeds and
matches full ordered Native bytes in both modes using only generator allowance.
Painted exits1 with no G-code:
`InvalidInput("invalid project model XML: attribute namespace does not match its 3MF meaning")`.
The Model XML allowlist rejects canonical paint_fuzzy_skin before serde/slicing;
Triangle has no fuzzy carrier field. No target equality or expected-error PASS
claimed. API calls0; this is discovery, not public RED.

Initial driver stopped after the first successful Native reference because the
activity parser default expected an explicit T selection. Continuation uses its
already-existing initial_tool=0 parameter for this one-filament stock project,
reuses that exact raw Native/process receipt without rerun, then completes the
remaining7 Native and2 CLI. Original script/log/exit1 and continuation exit0
retained. This is an observer stop, not unstable Native or semantic gate credit.
CPU18/jobs1/thread controls1; actual Native TIDs measured separately, not called1.
A byte-frozen copy of qualified70e51b4e2 CLI was reused; core/CLI source/lock tree
identities equal92ce9b02c. No fresh exact-tip build or API qualification claim.

## Included complete consumed behavior for later authorized GREEN

- Recognize only canonical unprefixed paint_fuzzy_skin without weakening other
  XML namespaces/unknown-attribute checks. Decode/carry real annotation data via
  the existing codec/shared mesh identity; no raw-string side channel, dropped
  split tree, frozen-coordinate cache or hardcoded face/state geometry shortcut.
- Derive actual source-owned fuzzy parent-region identities/configs in the same
  registry as ordinary/MMU regions. None becomes All for painted region config;
  Disabled_fuzzy stays Disabled_fuzzy. Preserve ordering, deduplication and parents.
- Port the fuzzy wrapper's2facet states, maximum real external-perimeter flow
  width, depth0, beam=false, IncludeTopAndBottomLayers::No. Resolve actual flow
  as upstream PrintRegion::flow does; do not substitute a nozzle/width constant.
  Do not reuse MMU's colour count, caps=Yes or painted/extruder remapping by fiat.
- Port actual apply intersections, bbox/parent/self tests, stolen/remaining slices,
  source opening/closing/merge and surface replacement. The same real graph and
  LayerRegion identities must reach perimeter generation; no private-phase-only
  producer, staging marker or duplicate region state accepted as completion.
- Preserve literal MMU → fuzzy → single existing interlocking → compensation
  order for the admitted domain. Do not move/duplicate interlocking or add a
  dummy fuzzy stage. Preserve downstream compensation and stock EFC0.1. Both
  XY options in this witness are0; do not infer a nonzero fuzzy XY suppression
  rule merely from the upstream warning text. That domain needs separate audit.
- Feed classic FuzzySkin.cpp561–684 compatible/merged region masks into the
  existing classic materialization seam. Retain source should_fuzzify, loop/hole
  eligibility, deterministic Ripple scaled points and mask splitting/concatenation.
  Config-only whole-polygon perturbation is not an implementation of painted masks.

Initial public runtime domain: the frozen one-object/one-instance/model-volume,
one-filament classic/Ripple stock pair. Implementation remains a generic rewrite
of the cited functions, not a model-specific workaround. Existing consumers and
stock global-fuzzy controls must retain their meanings. Core is byte-oriented,
portable memory/data only; filesystem/terminal remain in adapters.

## Public RED after independent SPEC approval

Freeze these exact inputs; reuse shared NativeCase/native_stream_case and existing
comparator/process seams. Reference/replay/settings/positive model extrusion and
active painted-face diagnostic in both modes must precede every public Ares call.
Use the existing monofilament diagnostic initialization explicitly, not a new
process harness or a fabricated tool-selection command in the output stream.

Tests require real validated API AND real CLI success, full ordered per-input
Native/API/CLI comparisons and raw API==CLI. The current rejection must FAIL the
success assertion, not become an expected-error PASS. Preserve both API error and
CLI status/output on rejection. Unpainted must PASS. Then submit RED RR and wait
for explicit GREEN authority; no production edits before that approval.

Pre-code failure inventory: ignoring canonical attribute; admission-only patch;
wrong None/Disabled rule; codec/tree or shared-index loss; wrong wrapper caps/state
count/width/cast; staged-marker registry substituted for actual identities; paint
applied to the whole polygon; missing mask merge/split; duplicate interlocking;
wrong compensation gate; Native stochastic/replay/control/affinity failure;
header normalization expanded; fabricated success or platform/runtime claims.

Any Native/control/resource/replay failure STOPs as non-gate. Genuine algorithmic
residual outside the cited boundary needs STOP/RR, not backend/normalization fixes.

## Deferred / authority and exit

Defer MMU+fuzzy and fuzzy+beam inputs, Arachne-painted masks, support/raft/tower/
custom/multi-object expansion, nonzero XY-compensation combinations,
full stochastic/noise matrix and platform runtime.
Do not accidentally admit unsupported compositions merely by permitting the
attribute: retain fail-closed admission for those deferred inputs until separately
reviewed. Their future owning ParentType::PaintedRegion source remains cited,
not claimed qualified here. No new public phase API, pipeline, geometry backend,
registry, fallback or unrequested file I/O.

Exit stages: SPEC approved → exact-input success-required public RED → independent
RED approval and GREEN instruction → complete consumed slice → content review.
Later exact-tip fmt/strict all-target Clippy/WASM compile/directed regression and
canonical dual FULL require their assigned integration authority/window. Files
≤400LOC, no include tricks. No production/test edits or FULL in this docs request.
The frozen beam stack92ce9b02c and its W1 integration window remain independent.
