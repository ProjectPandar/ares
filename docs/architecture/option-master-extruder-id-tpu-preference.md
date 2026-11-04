# master_extruder_id — bounded TPU preferred-nozzle rewrite

## Offline audit and provenance

Audited Ares tree: `711b96bd256c2d780eac5dc4227427f9e2feb7f2`.
Upstream: OrcaSlicer 2.4.2 source commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, accessed through `OrcaSlicer/`.
The option audit classifies this key as OPEN_PLANNER, one baseline case, scope
incomplete. That historical INCOMPLETE is not a new execution result. This slice
does not select PCP, `timelapse_type`, `printer_technology`, or another key.

| Owning upstream boundary | Included meaning |
| --- | --- |
| `src/libslic3r/PrintConfig.cpp:5399-5403` | `master_extruder_id` is `coInt`, default 1; no declared min/max here. Do not invent a legal numeric interval. |
| `src/libslic3r/GCode/ToolOrdering.cpp:1107-1116,1150-1206` — `ToolOrdering::get_recommended_filament_maps` | Convert configured one-based master to zero-based. Only the two-nozzle BBL branch selects the TPU shortcut; select TPU from actually used layer filaments, not all configured material types. |
| `src/libslic3r/FilamentGroup.cpp:40-50` — `calc_filament_group_for_tpu` | Produce a vector for all configured logical filaments: TPU-set members go to master; all other members go to `1-master`. |
| `src/libslic3r/GCode/ToolOrdering.cpp:1235,1279-1313` — `ToolOrdering::reorder_extruders_for_minimum_flush_volume` | Auto modes invoke recommendation; Manual bypasses it. Convert recommendation back to one-based and update print config. Check the resulting material/nozzle printability. |
| `src/libslic3r/Print.cpp:3166-3193` — `Print::update_filament_maps_to_config` | Map update is not just a footer edit: it refreshes grouped filament variants and retraction overrides. Existing emission must consume the effective map. |
| `src/libslic3r/Print.cpp:3216-3249`; `GCode/ToolOrdering.cpp:67-82` | Physical printability constraints and post-group check remain intact. No guard removal to admit an input. |

Important negative finding: the ordinary non-TPU dual-BBL route calls
`FilamentGroup::calc_filament_group` (`FilamentGroup.cpp:578-591`), including
Match mode and Flush mode optimization (`:755-774`). Initializing a vector to
master is **not** proof of the final ordinary PLA assignment. This spec therefore
does not claim the broader Auto grouping algorithm or use a two-PLA master flip
as a source-proven expected result.

## Rust boundary and temporary scaffold

Existing `crates/ares-core/src/options/filament_map_recommend.rs:16-39` is a
compatibility shell: for dual BBL it assigns every configured filament to master,
without the used-layer TPU set. It is not a completed `ToolOrdering` port.
`crates/ares-core/src/project/effective_config/phases.rs:117-133` currently
snapshots the pre-recommendation map and calls this shell before emission;
upstream recommendation runs on actual `m_layer_tools` after validation.
These phase/usage differences are disclosed, not renamed into a new Ares-owned
pipeline or silently generalized away.

The intended core destination is the existing source-owned recommendation module,
with the bounded `calc_filament_group_for_tpu` operation kept separate from the
deferred ordinary optimizer. Its inputs must represent upstream configured
filament count, the **actually used** TPU logical-ID set and zero-based master.
It must not guess actual usage from the presence of a configured `TPU` string.
The fixed admitted fixture must show both logical filaments actually extruding.
A native-only config comment or a Rust helper test cannot supply that evidence.

**Production seam gate:** no edits to the G4 sequence/footer files or any
shared hook. In particular do not modify `phases.rs`, shared
option-domain dispatch/sweep/runner hooks, G4 fixture helpers, or footer writers
under this spec alone. If legal runtime RED needs a used-layer handoff or a map
refresh at one of those seams, submit the exact files, upstream call timing and
proposed ownership split before touching them. Do not add a
parallel scheduler, broad material scan, fallback mapping or unchecked bypass
instead. The narrow pure operation alone does not close runtime parity.

Harness destination: new standalone
`crates/ares-cli/tests/orca_parity/option_coverage/master_extruder_id.rs`, or a
dedicated registration seam. No helper visibility changes or shared
routing edits. Existing G4 inputs
may be read as shape references, but their builders remain untouched.

## Included bounded witnesses

Use one plate, by-layer cube10, two 0.4 mm Standard nozzles on BBL H2D, relative E,
no tower/support/raft, two coherent logical materials TPU and PLA, wall1 and
sparse/solid2 with actual extrusion by both over multiple layers. All per-nozzle
selectors, variant identity, material counts, map shapes, plate mode and both
materials' recommended temperature compatibility must be coherent. Freeze those
dependencies before execution; do not carry the old G4 singleton-selector shape
or blindly reuse its 211/233 temperature pair. No new stock preset admission.

| Case | Mode / master | Source-expected effective map |
| --- | --- | --- |
| A1 | Auto For Flush / 1 | `[1,2]` for used TPU0, PLA1 |
| A2 | Auto For Flush / 2 | `[2,1]` for the same used-set |
| M1 | Auto For Match / 1 | `[1,2]`; TPU shortcut precedes ordinary Match solver |
| M2 | Auto For Match / 2 | `[2,1]` |
| C1 | Manual / 1, input map `[1,2]` | `[1,2]`, master inactive |
| C2 | Manual / 2, same input map | `[1,2]`, master inactive |

These are finite context witnesses, not admission of arbitrary integer masters,
TPU mixtures, palette cardinalities or all Auto maps. An unused configured TPU
countercontrol is needed to prevent broadening the TPU trigger; its ordinary
non-TPU final map is not asserted from this audit. Retain its Native observation
and do not port the optimizer to make that countercontrol pass within this slice.
If source-correct routing of that countercontrol needs the deferred optimizer or
a shared seam, stop for a scope decision rather than weaken it.

The Manual pair may differ in serialized `master_extruder_id`; require Native/Ares
whole-output equality for each input, not equality of C1 and C2 files. For active
pairs, prove Native map recomputation actually overrode an identical frozen input
map. Show logical filament and mapped nozzle use at real wall/infill extrusion;
existing stock commands and source placeholders are witnesses, not a comparator
filter. Footer-only variation is insufficient.

## Test-first admission and implementation gates

1. Perform Native
   admission with complete identical-input bindings in seeded and official modes.
   Save final ZIP/member SHAs, requested/loaded project and plate settings, material
   and selector vectors, exact command/env/binary identity, stdout/stderr/status,
   result JSON and full G-code. An export/reload may overwrite mode or palette;
   prove final values and Native effective map rather than assume application.
2. Require successful legal Native slicing and actual active-branch/nozzle
   witnesses before calling a result production RED. Required Native absence
   fails closed; optional absence is an explicit skip, never Native proof.
   Any printability, material-temperature, export, palette or UB-adjacent failure
   remains an admission blocker. No retry fixture mutation, guard removal or
   output tolerance.
3. Commit standalone E2E RED before production. Compare the real validated public
   API and actual CLI against Native on the same project. Separate metadata,
   application, admission and production failures. A genuine mismatch needing
   production or a shared seam requires a scope decision first.
4. Implement only the source operation and its adapter boundary.
   No production correction is promised before legal RED. If all legal cases
   already match, keep the slice harness-only rather than manufacture a failure.
5. Canonical plan admission is a subsequent key-local decision, not implicit:
   retain existing baseline and every other plan unchanged. Do not globally mark
   this integer domain complete or replace the baseline fixture with these Auto
   contexts.
6. For an implementation, run new-tree `cargo nextest run --workspace`,
   workspace all-target/all-feature strict Clippy, fmt and core/WASM adapter checks.
   Fresh seeded/official base/candidate controls
   must retain the exact failure-name subset rule, compare all common complete
   outputs and disclose raw changes, multiplicities and extra Native captures.
   Ignore only entire generated-by G-code lines; retain existing CLI wall-clock
   exemptions only. No numeric, ordering or footer tolerances.

Failure inventory before coding: input mode/palette overwritten by Native export;
TPU profile rejected before recommendation; selector cardinality UB; only one
material actually used; master one/zero-base confusion; shell ignores TPU
complement; Manual recomputed accidentally; configured-but-unused TPU triggers
shortcut; recommendation occurs at wrong phase or variants are not refreshed;
Native effective map is not observable; unrelated complete-output mismatch.
Every failure blocks the corresponding claim.

## Deferred and non-negotiable decisions

Deferred: ordinary `FilamentGroup` Match/Flush/PAM/AMS optimization and its existing
exception path; TPU physical-printability rejection coverage; unused/mixed/painted
or arbitrary palette geometries beyond the trigger countercontrol; dynamic maps;
by-object ordering; single-nozzle master values other than existing baseline;
non-BBL multi-nozzle reassignment; SEMM/Type2/tower insertion; ramming; variant
reconstruction and dataset selectors; physical-extruder-map domain closure;
sequence/footer rewriting; timelapse, PCP and printer-technology admission.

This is a source-cited `libslic3r` rewrite slice, not a new Ares grouping design.
No legacy fallback. Core remains platform-neutral on WASM, Windows, macOS and
Linux; filesystem/Native fixture work stays in CLI tests. New files stay below
400 LOC. Do not touch the already oversized `project_slice.rs`; a later required
touch must first split that module.

Exit: legal admitted branch controls, committed test-first evidence,
source-owned correction (if needed), complete-output parity and engineering
controls, with all deferred behavior explicit.
