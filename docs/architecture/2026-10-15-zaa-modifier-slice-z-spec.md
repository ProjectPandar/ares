# ZAA follow-up survey and modifier-region slice-Z SPEC

Status: SPEC-REVIEW-REQUEST; no new Native or Ares run, no production edit.
Base: approved ZAA GREEN e34e8eed3db406ee862d7ec5d9ba1d59b2bc1845,
queued for stack-10 on main29f4a49c18139925fec238d31d2fdb6b790fbc30.
This docs-only branch leaves the approved candidate unchanged. Rebase onto
stack-10's eventual main before RED; keep all earlier GREEN witnesses.
Upstream inspected: OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1.

## Read-only survey: evidence versus hypotheses

Existing measured evidence is the classic single-object sloped roof only:
Native official/seeded69/69, including live contour output and disabled raw
control. Neither modifier nor multi-object nor Arachne ZAA output has been
measured here. The following are source differences or missing witnesses,
not newly established runtime REDs.

| Combination | Upstream ownership | Current Ares boundary and finding | Next disposition |
| --- | --- | --- | --- |
| Modifier-only ZAA activation or smaller modifier minZ | Print.hpp:423-424; PrintObjectSlice.cpp::compute_slice_z:24-47 | planning.rs::zaa_min_z:73-80 reduces only layer_candidates.model_parts; modifier-derived all_regions are absent from this reduction. contour_z::prepare/entities already use canonical regions. | First proposed slice below; source-level domain mismatch, pending public Native witness. |
| Multiple objects, enabled/disabled mix | PrintObject.cpp::need_z_contouring/contour_z:748-804; Model.cpp::raw_mesh:1518-1528 | contour_z::prepare iterates prepared objects, builds each object's model-part mesh, and applies its region options. No obvious cross-object activation shortcut found. | Separate fresh witness after first slice; vary object order and distinct roof geometry. Do not claim a defect yet. |
| Multipart, translated/rotated instances | Model.cpp::raw_mesh:1518-1528, min_z/update_min_max_z:1460-1498; PrintObject.cpp:768-782 | mesh.rs uses model-part-only vertices, per-volume transform, print-group transform and centering. Native uses first ModelInstance and explicitly rejects instance count !=1 at contour stage. Group/cloning semantics need traced admission before any expected-success case. | Deferred; exclude modifiers from ray mesh. Never assume that multiple instances are Native-admitted. |
| Arachne closed/open variable-width walls | PerimeterGenerator.cpp::traverse_extrusions:370-574, especially528-566; ContourZ.cpp:162-243 | arachne/variable_width.rs emits constant-width ExtrusionPaths; traverse.rs wraps loops/multipaths; contour_z/entities.rs visits their paths. No ZAA-specific generator rejection found at this seam. | Deferred public thin/tapered-roof witness with active width variation, XYZ and unchanged disabled output; investigate a real mismatch before a repair plan. |
| Nondefault fill-direction interaction | Fill/Fill.cpp:1241/1446/1631 assigns dont_alternate_fill_direction when both ZAA options are enabled | No use of zaa_dont_alternate_fill_direction in project_slice or fill implementations was found; the option is presently represented in the config domain. | Separate source-cited candidate, not folded into modifier slice-Z repair. Need Native acceptance plus direction activity and complete-output RED. |
| Scarf/sloped extrusion | ContourZ.cpp:185-188/209-212 throws for sloped entities | Combination is not an unconditional successful-output target. | Deferred Native rejection classification; no invented admission policy. |

## Owning upstream concept

PrintObjectSlice.cpp::compute_slice_z:24-47 computes one slicing plane per
physical object layer. It iterates num_printing_regions/printing_region,
chooses the minimum zaa_min_z among enabled regions, retains midpoint on
first layer or with no enabled region, otherwise uses lo+minZ. Print.hpp:
423-424 explicitly maps that iteration to m_shared_regions->all_regions.
This is an object-wide region-domain reduction, not a per-layer active-area
reduction and not a model-part-only option lookup.

Ares already materializes the relevant domain: effective_config/candidates.rs:
134-141 sets resolved_object.all_regions from object_region_graph.
region_graph.rs::add_modifier:137-175 resolves modifier settings against its
parent and registers the distinct child region. planning.rs:73-80 instead
uses the preceding model-part candidates. A modifier may therefore enable
contouring in entities.rs while the object slice plane still uses ordinary
midpoints, or may request a smaller minZ that the plane ignores.

ContourZ.cpp::Layer::make_contour_z:235-243 applies ZAA separately to each
region's paths. Model.cpp::raw_mesh:1518-1528 includes model parts only;
modifier geometry partitions regions but must not become ray-query geometry.
PrintObjectSlice.cpp:336-427 owns complex region clipping/assignment; this
slice consumes the existing Ares region graph without redesigning it.

## Rust destination and included behavior

Production repair, only after separately reviewed public RED:

- project_slice/planning.rs::zaa_min_z reduces enabled RegionOptions from
  ResolvedProjectObject.all_regions, matching the upstream iteration domain.
- Keep layers.rs::planned_layers_with_zaa:243-287 as the existing upstream
  compute_slice_z compatibility shell: first-layer midpoint, lo+minZ and
  existing bad-minZ error remain unchanged.
- Keep per-region contour application, model-part-only IndexedMesh and its
  deterministic Tree3f unchanged. No new mesh, region lookup, per-point copy,
  validation rule, Unsupported guard or fallback is introduced.
- Include modifier-only activation and enabled-parent/smaller-child minZ,
  with disabled/inherited-equal controls and one disabled neighboring object
  to prove option isolation within this bounded witness.

Deferred: general multi-object ordering changes, instance grouping/cloning,
Arachne segmentation, overhang wall processing, fill-direction alternation,
ironing-specific, scarf/arc, support/raft and invalid minZ policy. If RED
reveals a defect outside the reduction boundary, STOP and submit a repair
PLAN rather than expanding production scope. Existing Ares wrappers remain
compatibility shells around PrintObject/PrintRegion, not a new pipeline.

## Public E2E RED plan and admission conditions

After SPEC approval and an open directed window, CPU13 official/CPU14 seeded,
outer taskset, jobs2/threads2, each leg fresh cache/root/target:

1. Reuse the admitted roof geometry and compatible-regions modifier fixture
   construction. Change model geometry only into the proven sloped roof;
   bound a parameter modifier across a real, nonempty roof region. Preserve
   printer/filament settings, single-instance object and ordinary controls.
2. Cases: all-ZAA-disabled; parent enabled minZ=0.05 with inherited-equal
   modifier; parent disabled with modifier enabled minZ=0.05; parent enabled
   minZ=0.05 with modifier enabled minZ=0.02. Use valid fixed 0.2mm layers
   with no raft; verify the exported settings rather than assuming them.
3. Confirm both canonical Native modes accept the exact input, finish their
   config footer and repeat with fresh roots. Prove modifier intersection
   and distinct resolved options from loaded public project/config data.
   Require positive XYZ contour moves in its roof region. For smaller-child
   minZ, require a changed Native eligible-role stream against inherited-equal
   control. Save input/config/role SHA evidence and bounds in case artifacts.
4. Compare the entire Native output against public Ares API and CLI with
   the unchanged strict comparator; assert API/CLI raw equality. Keep all
   successful controls. Include a second separated, disabled object only
   if its disabled baseline first matches Native; record each object's role
   bounds so a global-on witness cannot masquerade as modifier isolation.
5. Commit the genuine public failure independently and report RED. If Native
   rejects/crashes, the modifier is empty, no contour activity is observed,
   or the purported mismatch cannot be separated from an ordinary baseline
   defect, STOP and report. Do not fabricate RED using private helper or
   plan-entry assertions. If all cases match, retain regression controls and
   report the disproved runtime hypothesis before any production change.

After RED approval, make the bounded reduction repair, preserve RED assertions,
then run those cases both modes plus existing ZAA/compatible-region controls,
strict clippy/fmt/wasm. Production/module files remain <=400 lines; split new
E2E fixture/proof files only as needed. Native artifacts are external and
repeatable, never committed. Full/workspace/CI/merge belong to coordinator.

## Non-negotiable decisions

- Source all_regions semantics takes precedence over a model-part shortcut;
  do not filter by current layer occupancy or replace per-region ZAA with a
  global switch. No regions[0], new unsupported guard or silent skip.
- Existing ZAA public RED/comparator/disabled-byte controls stay unchanged.
- This is a source-cited PrintObjectSlice rewrite correction. Coverage gaps
  in the survey do not independently justify unrelated implementation work.
- No Native/build/experiment was run for this SPEC; prior measured GREEN
  does not establish the pending modifier, multi-object or Arachne outcomes.
