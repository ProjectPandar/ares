# C2 dependency: Native import identity and selected-load contracts

Status: Proposed. Further implementation waits on this contract.
Prototype: a07c7c3cb732b3ffeb6ea5970a9bcc387bf55423.

The C2 spec required: “公共 load/selection 契约改动须另 SPEC”. Two
Native-backed selection witnesses and the surviving literal-ID mismatch now
show why that condition applies. C2 is **not GREEN**: both directed modes have
36/37 PASS, with `c2_nested_bfs` deterministically failing. All fourteen C1
controls pass. No retry, normalization, fixture retirement or new production
change has been used to conceal these failures.

This dependency ports `_BBS_3MF_Importer` model identity/materialization and
the CLI's selected import prefix. It does not add an independent pipeline.
Source tree: OrcaSlicer 2.4.2 seed-replay source.
libslic3r references are relative to `src/libslic3r`.

## Source boundaries and concrete differences

| Source | Native behavior and current Ares difference |
| --- | --- |
| `Format/bbs_3mf.cpp:744,920` | Import IDs are `(raw string,int)`; they are not authorized/normalized archive PackagePath identities. |
| `Format/bbs_3mf.cpp:1771-1778,3661-3663` | The main-model object namespace is the empty submodel path. Submodel paths are used during their import and then cleared. |
| `Format/bbs_3mf.cpp:3851-3872`, `_handle_start_component` | For the main model, the component's literal p:path supplies the ID's path; no path attribute gives the empty main namespace. |
| `Format/bbs_3mf.cpp:5564-5584`, ObjectImporter component handler | Submodel components use their importing object_path rather than the component's p:path. |
| `Format/bbs_3mf.cpp:4867-4891`, `_generate_current_object_list` | BFS looks up those exact IDs. Only a found object expands components or contributes geometry. There is no canonical-path fallback for an absent importer ID. |
| `Format/bbs_3mf.cpp:2058-2082`, `_load_model_from_file` | A positive requested plate selects obj_inst_map membership by object integer ID before volume preparation; absent objects never prime m_shared_meshes. Plate0 does not apply this importer filter. |
| `OrcaSlicer.cpp:1640` | The CLI passes plate_to_slice into Model::read_from_file, so this scope is chosen during import, before later placement and slice selection. |
| `Format/bbs_3mf.cpp:5025-5052` | First available mesh is selected within that actual import scope, with the accepted fresh/reused shift distinction. |

Ares's `load/graph.rs` authorizes and normalizes archive references, then
`assemble/component_graph.rs` and `assemble/volumes.rs` traverse those package
identities. That is a useful package-validation shell, but it is not the
Native importer identity. A main-model component explicitly pointing to
`/3D/3dmodel.model` can resolve in Ares while missing Native's empty-path ID.

`domain/plate_selection.rs` currently maps None to the first plate, retains
unassigned instances for the geometric gate, and filters assigned instances.
The prototype additionally rematerializes based on `plates.len()>1`. Neither
rule expresses Native's all-import versus explicitly selected-import scope.
`project_slice/profile_entry.rs:73-85` also applies placement before its later
state selection; an explicit scope cannot be deferred until after placement
if it changes the loaded bounding box.

## Evidence: original failures and every setup retained

The tested head is c704edebf.
The parent RED4334347e4 remains intact; initial strict import errors and their
test-only cleanup also remain recorded. These new preflights are geometry and
admission witnesses, **not strict parity PASS counts**.

1. Original committed `nested-bfs.3mf`, both directed modes:
   Native exit0, triangle_count12,18 positive first-layer wall endpoints,
   X[164.31,179.69]. Ares API/CLI admit and agree with each other, but have
   36 endpoints and X[119.31,179.69]. Full ordered comparisons fail in both.
   The direct leaf is present; the literal explicit-self-path branch is not
   found in Native's main-model ID namespace. The case and every assertion
   remain in Git unchanged.
2. Local-BFS preflight:
   change **only** that component's p:path from explicit self-path to absent.
   Native official/seeded exit0,36 endpoints, X[119.31,179.69]. Prototype
   matches those bounds; the preserved C1 producer gives X[122.31,179.69].
   This independently exercises actual local-ID BFS order and shared-owner
   geometry. It does not replace or void the original failing input.
3. Unassigned-owner preflight:
   start with committed implicit-shared input; only make owner object2
   nonprintable and remove its plate1 instance metadata. Geometry, settings,
   IDs and the printable later object stay unchanged. Both Native modes and
   preserved C1 give Xmax179.69; prototype gives176.69. All producers exit0.
   This is a new3mm candidate regression caused by an excluded owner priming
   the all-object cache, not Native instability.
4. Default-scope preflight:
   start with committed two-plate input; swap object/label plate membership
   and put the owner at globalX374 on plate2, later object atX155 on plate1.
   Both Native modes give first-plate Xmax176.69 for --slice0 and179.69 for
   --slice1. Prototype default and explicit --plate1 both give179.69.
   Thus default/all-import and explicit/plate-import cannot share one scope.

Each preflight uses fresh per-producer cache/artifacts.

## Public semantics

Keep public signatures, borrowed mesh accessors, settings/custom-G-code
selection, plate numbering and error types. `load_project(bytes)` remains an
unselected/all-import model. A slice with None still emits the first plate,
using the original all-import owner's geometry, corresponding to Native
--slice0's first output. On an already selected view, None retains that
view's prepared scope and plate. An explicit Some(id) chooses the Native plate-import owner set
before geometry preparation and placement.

For an explicitly selected import, objects absent from that plate's Native
object-ID membership are excluded, including unassigned/nonprintable owners.
This deliberately changes the current public selected-view membership; it is
the precise selection-contract change specified here. Preserve the existing
assigned-instance view filtering, world coordinates and original-source
provenance needed by C1. A selected view remains restricted to its available
plate; this does not allow selecting a removed plate or resetting origins.

Represent the scope explicitly in private import provenance, rather than
infer it from the number of plates. Already materialized/placed views must
not be centered or normalized twice. A,B,A calls on the same original loaded
Project remain independent and preserve A exactly.

For profile/--load-settings requests with an explicit plate, apply the same
object import scope before the existing placement prefix. Preserve the full
plate metadata needed for global grid origin and later state selection;
do not call the final one-plate view selector twice or reset curr_plate_index.
`profile_placed_project`'s explicit-plate object set consequently becomes the
Native scoped loaded model instead of today's all-object placed model. This
observable change is also intended; unrelated profile/settings resolution,
compatibility and validation error ordering are outside this SPEC.

`load_project` volumes must reflect Native importer IDs, including the
original nested input's one imported volume, rather than two normalized-graph
leaves. Keep package authorization/validation separate; do not reinterpret a
missing archive file, malformed transform or unknown option as an absent
Native ID. No legacy canonical-ID fallback is permitted.

**No-silent-skip rule:** only the exact source-cited absent-ID branch of
`_generate_current_object_list`, with this Native-admitted witness, may skip.
This is an importer materialization contract, not permission to skip a
feature, oracle comparison or test. Do not manufacture geometry or an
Unsupported guard to avoid the discrepancy.

## Rust boundary and included/deferred behavior

Retain normalized PackagePath graph authorization as the input shell. Carry
Native raw model/component IDs alongside it into private loader occurrence
materialization. Build Native's found-ID BFS leaf order before metadata
matching and integer-key mesh reuse. Avoid hardcoding the self-path fixture.
Private scope handling spans load/assembly, domain selected views, and
project_slice's existing load/placement prefix. All logic remains in core
without filesystem access or Native processes.

Included: main/local/submodel ID semantics, accepted absent-ID materialization,
all versus explicit plate scope, excluded-owner isolation, existing placement
prefix on explicit requests, one-time shift normalization and C1 frames.
Deferred: changes to archive security/resource limits, unrelated config or
error-order admission, arbitrary plate reselection from a restricted view,
export/reload/undo, new algorithms and any unrelated feature guard.
If the materialization port requires altering those deferred validation
contracts, stop and report that additional boundary before changing them.

## TDD, assertion corrections and release

Before further production, commit public Native-backed RED for unassigned
owner, default-vs-explicit owner placement and literal-ID volume membership.
Add the local-ID BFS input as an additional control; do not remove the old
explicit-self-path input. Both modes retain strict complete output comparisons.
Public loaded/selected geometry and profile placement are separately observable
seams; exercise a relocated explicit plate and A,B,A on one loaded Project.

Correct only the original nested fixture manifest's
expected imported volume_count from2 to1, based on Native triangle_count12
and the cited BFS branch. Keep its equality assertion and ordered comparator
fully intact; this corrects a hand-authored Ares-graph assumption, not Native
output. Every other existing assertion stays. Any further obsolete contract
test needing changes must be listed separately before editing it.

Then implement this import-contract dependency in its own production commit,
with no change to the specified fresh/shared offset algorithm. Keep files at
most400LOC, no probes/allow/ignore/regions[0]/normalization. Directed runs use
fresh roots per run. Strict/fmt/wasm and the complete C2+C1 matrix must pass.
