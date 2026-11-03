# SPEC C2-S: local internal-solid Grid admission

Status: SPEC-REVIEW-REQUEST; source/documents only, no RED/production edit/run.
Base `a120ebd5ce18a7c4e51c7502c2bc1faa8324cb06` (C2-T merged).
Worktree `${TMP_ROOT}/ares-worker2-bridge-anchor-c2s`, branch
`worker2/bridge-anchor-c2s`. C2-S requires its own approval, RED, GREEN and REVIEW.

## Owning upstream boundary

Orca source commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, libslic3r:

| Source / function | Behavior ported or retained |
| --- | --- |
| Config.hpp:2084-2092, ConfigOptionEnumGeneric::deserialize | Local enum decode uses the complete keys map, so Grid is a decodable local value. |
| PrintObject.cpp:3602-3659, apply_to_print_region_config | Applies known local PrintRegionConfig options. |
| PrintObject.cpp:3664-3709, region_config_from_model_volume | Merges object/model-part/material/range fields and normalizes density/filament/fuzzy settings, without enforcing the global solid-pattern subset. |
| Fill/Fill.cpp:858-902, group_fills | Ordinary solid infill selects internal_solid_infill_pattern at :891-893 and density100; bridges take their separate monotonic/rectilinear branch. |
| Fill/Fill.cpp:1394-1446, Layer::generate_sparse_infill_polylines_for_anchoring | Only stInternal surfaces contribute sparse anchors (:1403-1406); Fill dispatch uses their sparse params. |
| PrintObject.cpp:2735-2754, PrintObject::bridge_over_infill | Generates sparse anchors on required lower layers. |
| PrintConfig.cpp:2106-2112,10416-10419 | Global internal-solid has the eight-value top domain and is checked before slicing. Global Grid remains invalid. |

This ports the local PrintRegionConfig/Fill distinction, not a new Ares
supported-input policy. Global C2-T validation and exact legacy canonical
validation remain unchanged; local Grid must never be admitted by widening
the global schema.

## Rust destination and parallel ownership

Initial production destination only:
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs`.
Its internal-solid eight-value gate rejects a known local enum before sparse
anchor preparation. This validator remains a temporary compatibility shell
around upstream bridge/Fill concepts. The minimal proposed repair is to
delete that single role gate and its newly unused ProcessInfillPattern import,
after establishing admitted public RED; numeric guards remain for D.

Existing consumers remain unchanged: `options/region_options/metadata.rs:21-44`,
`overrides.rs:32-41`, `merge.rs:15-51`, `normalization.rs:4-23`, and
`project_slice/group_fills/params/projection.rs:64-70` already select local
ordinary-solid pattern separately from bridge pattern.

W6 owns the Grid arm in `bridge_over_infill/sparse_anchoring/arms.rs:56-75`
and the Grid-related candidate predicate/box work in
`bridge_over_infill/transaction/candidate_expansion/layers.rs:65-78`.
C2-S will not edit either file, their callers or shared Fill algorithms.
Fixtures keep sparse_infill_pattern=rectilinear, density20 and top pattern
valid, varying only local internal-solid metadata. The chosen Grid value is
for ordinary solid fill, not sparse anchoring. If an actual mismatch requires
any W6-owned change, stop and report a source-cited PLAN/ownership request.
Do not expand production scope based on the shared enum spelling.

Included: public object/model-part internal-solid admission witness,
selected-role guard removal, whole Native/API/CLI comparisons and controls.
Deferred: other local solid patterns, sparse Grid/arm/bounds changes, density,
rotations, numeric anchors, surface-kind redesign, modifier/material/range
combinations, Fill algorithm changes, brim/counterbore and other legacy work.
If deleting the gate reveals a Fill mismatch, preserve RED and request a new
PLAN before changing any consumer or assertion.

## Public Native preflight and RED plan

Only after this SPEC approval and an open directed window: official CPU13 /
seeded CPU14; outer cargo/nextest/Native pinned, jobs2/threads2; independently
fresh Native cache and artifact root each invocation. No full/workspace.

Export a public vendor-profile cube/solid-ceiling 3MF. Keep global
internal-solid=monotonic, top=monotonicline, sparse=rectilinear/density20.
Insert exactly one local internal_solid_infill_pattern key into
Metadata/model_settings.config at object or normal model-part scope. Verify
that exact key/value/scope; SHA-check all other ZIP members unchanged after
injection. Write the final public bytes to the on-disk Native input and feed
the same bytes to the public Ares API and CLI. No internal record injection.

Finite cases:

1. Active object Grid on the solid ceiling, Native Internal Bridge moves>0.
2. Active model-part Grid on the same ceiling, same activity witness.
3. Matched local rectilinear control for each scope/geometry, otherwise
   identical input: Native and Ares complete output must match.
4. No-candidate geometry control: a short solid body whose shell layers
   overlap, with sparse setting still20. Initial public geometry candidate
   is a 1.4mm body (cube10 vertex Z scaled0.14 before Native export), top5 /
   bottom3. Require Native Internal Bridge moves=0 and nonempty Internal
   solid infill moves; this is a measurement, not an assumed control.

Native must emit a nonempty complete stream (CONFIG_BLOCK_END), with active
Internal solid infill extrusion moves. Compare Grid versus rectilinear
selected-role move SHA/count and whole non-header bytes to prove consumption;
a global config footer is not a local override witness. Never relabel a
rejected/crashed Native case or absent role as RED. If the active witness or
no-candidate control cannot be established, stop/report the actual evidence.

For admitted active Grid, observe exactly public
UnsupportedProjectFeature("internal_solid_infill_pattern") on the baseline.
Permanent expectations remain whole-output generator-only PASS and CLI/API
raw equality; baseline must fail at that expectation. Save public input,
Native/API/CLI streams and errors, status, raw SHA, launcher/producer receipt,
role moves and activity before assertions. Commit public RED separately and
report it before repair; no assertions are changed after production edits.
Reuse the existing global-solid Grid rejection regression unchanged.

## GREEN, review and scheduling

After RED review and repair authorization: delete only the selected gate /
newly unused import. Preserve grouping, role/pattern/rotation, error and
write-back order. Unchanged finite assertions must pass both modes, followed
by bridge/anchor regressions, option_coverage and strict/fmt/wasm32.
Files stay <=400 LOC. No Unsupported guards, silent skip, ignore, suppression,
debug output, regions[0], weaker comparison or committed Native artifact.
Submit C2-S code REVIEW separately; heavy gates need explicit allocation.
Current directed window ends01:40, then W3 C1 exclusive; execution waits for
approved SPEC and an open window. Main merge/push remains coordinator-only.
