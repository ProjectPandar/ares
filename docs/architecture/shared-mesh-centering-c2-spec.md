# C2: bbs_3mf shared mesh preparation

Status: SPEC-REVIEW-REQUEST; documentation only, no C2 implementation or runs.
Base: merged C1 `4ca7554c6e729528883f9860ca700f874aeade00`.
Branch: `pi/worker3-shared-mesh-centering-spec`.

This independent slice refines the C2 boundary in the approved
`print-object-shared-mesh-centering-port.md` (original SPEC49cf8a57a).
C1 is closed: its canonical official/seeded full each passed1155/1155.
C2 requires its own public Native RED, GREEN, code review and allocated full.
This is a Rust rewrite of the bbs importer and ModelVolume preparation, not
an Ares-owned slicing pipeline.

## Owning sources

References use OrcaSlicer2.4.2 at
`/nix/store/hxvv0g1qlw9lwimj2km3ghxphrl9p4qf-orca-slicer-2.4.2-seed-replay-source`,
relative to `src/libslic3r`.

| Source / function | Required behavior |
| --- | --- |
| `Format/bbs_3mf.cpp:744,920,2058-2082`, importer object assembly | Iterate the ordered `(path,int id)` object map; exclude objects outside the requested plate before preparing their volumes. |
| `Format/bbs_3mf.cpp:4867-4891`, `_generate_current_object_list` | Breadth-first component expansion, preserving sibling order and accumulated transforms. |
| `Format/bbs_3mf.cpp:4904-4949`, `_generate_volumes_new` | Match leaf metadata by position then subobject id. Default sharing key is the leaf integer id; explicit `mesh_shared` uses `atoi`. |
| `Format/bbs_3mf.cpp:4955-4974,5025-5035` | A load-local integer-keyed map chooses the first available prepared mesh. For `-1`, lookup and register under the subobject id. An unseen explicit key uses the current mesh. |
| `Format/bbs_3mf.cpp:4976-5023` | Fresh geometry validates indices, retains its triangles, corrects negative signed volume, then creates a fresh ModelVolume. |
| `Model.cpp:1241-1265,1295-1307`, `add_volume` / `add_volume_with_shared_mesh` | Fresh creation centers once; shared creation reuses the centered mesh without recentering. |
| `Model.cpp:2600-2617`, `center_geometry_after_creation`; `TriangleMesh.cpp:271-284`, `translate` | Compute the mesh center, subtract float components from vertices, retain `init_shift`, and move the fresh volume offset. |
| `Format/bbs_3mf.cpp:5037-5052`; `Model.cpp:2821-2824`, `ModelVolume::translate` | Apply component transform first, then add the reused mesh's `init_shift` directly to the volume offset. |
| `Format/bbs_3mf.cpp:5054-5105`, volume annotations and metadata | Per-occurrence facet annotations, type, name and source metadata belong to the current volume, not the cached owner. |
| `Format/bbs_3mf.cpp:2187-2207`, single-volume offset redistribution | After assembling the effective loaded subset, move its volume offset into every source instance once. |

## Existing scaffold and Rust destination

`project/load/volume_metadata.rs` currently stores only presence of
`mesh_shared`. `project/load/assemble.rs` prepares every leaf independently in
build order. `project/load/mesh_prepare.rs` centers a fresh mesh and folds its
shift into the component transform, without retaining a reusable init shift.
`project/domain.rs` carries a private bool on ProjectVolume.
`project_slice/raw_intersections.rs::validate_shared_meshes` rejects that bool
or repeated `(source_model_path,id)`; cross-file equal integer ids escape it.
These are compatibility shells to replace, not fallback paths to retain.

The destination is the existing platform-neutral project loader/domain:

- `load/assemble.rs` and small private assembly modules: ordered occurrence
  collection and first-owner registration, independent of output/build order.
- `load/volume_metadata.rs`: signed key semantics instead of a presence bool.
- `load/mesh_prepare.rs`: centered immutable geometry plus its init shift,
  separated from occurrence transform and annotations.
- Private Project/ProjectVolume provenance and
  `domain/plate_selection.rs`: materialize sharing for the effective import
  subset from immutable original occurrence data.
- `domain/volume_offsets.rs`: reuse the existing offset redistribution after
  materialization; never redistribute already-normalized instances twice.
- `raw_intersections.rs`: remove only `validate_shared_meshes` and its call
  once the loader semantics and public strict comparisons are GREEN.

`assemble.rs` is403LOC at this base. Before any production growth, move its
existing component traversal/budget/cycle-validation responsibilities into
private files in a separate, purely mechanical refactor commit. Preserve
existing statements, ordering, errors and assertions; verify the expanded
source reconstruction and existing public controls. Every resulting file and
every later implementation file must stay at most400LOC.

## Required semantics

Cache identity is the effective signed integer key, not path, pointer identity
or an explicit-sharing flag. Preserve `atoi` behavior for representable
inputs, including sign, whitespace, decimal prefix and no-digit zero; do not
replace it with strict Rust integer admission. Overflow outside C int's
defined conversion domain is not a new supported input contract.

Prepare/register the first eligible mesh in Native's sorted object and BFS
leaf order, while retaining Ares's existing source/build output and instance
order. A later occurrence can have different raw vertices; its raw geometry
must not replace or recenter the owner. Keep each occurrence's metadata and
annotations separate from shared vertices/triangles. Immutable handles may
represent Native's shared mesh; public borrowed slice accessors remain intact.

Fresh and reused transform restoration are deliberately different. If `C` is
the accumulated component transform and `s` the owner init shift, fresh
creation uses `C * Translation(s)`. Reused creation first installs `C`, then
adds `s` directly to its offset (`Translation(s) * C`). The current
`Transform3d::translated` performs the former and `pretranslated` the latter.
Do not route both branches through the fresh-mesh helper. Keep floating-point
operation order and the existing fresh winding/f32 centering behavior.

Selected-plate materialization must start with original occurrence geometry,
component transforms and pre-redistribution source instances. Filter the
effective object subset before choosing owners; normalize volume offsets only
after that view's mesh reuse is resolved. An excluded object cannot prime the
map. Calls selecting A, B, A from the same immutable loaded Project must give
the same A output twice and agree with independent Native imports of each
plate. The unselected project may use an all-object map, but that prepared
result cannot become the source geometry for a later selected view.

Included: default/explicit/`-1` keys, cross-file integer collisions, repeated
component leaves, unseen owner keys, import/BFS ordering, fresh vs shared
shift restoration under rotation/scale, plate-local first-owner choice,
existing volume types/configuration and supported-facet annotation isolation.
Preserve C1 group frames and its fourteen controls.

Deferred: export/reload/backup/undo, arranging, SLA, native pointer identity,
new annotation algorithms or admission for otherwise guarded painted features,
and unrelated mesh repair. Existing option/feature guards outside this slice
remain unchanged. No new guard or silent independent-mesh fallback is allowed.

## Public contract boundary

Keep load entry points, `Project::select_plate(Option<u32>)`, first-plate
selection, error contracts, immutable input Project, public mesh accessors,
plate identity and settings/custom-G-code selection unchanged. Correcting
geometry in a selected view is the intended upstream rewrite behavior.
Original data retention and mesh sharing are private representations.

Do not introduce a new public deferred-loading API, change which plates are
selectable from an already-selected view, change current whole-package
validation/admission, or alter externally visible load/selection ordering.
If matching Native's plate-local importer requires any of those changes,
stop before implementing that dependency and submit a separate source-cited
public-contract SPEC. This document does not authorize it.

## Evidence and RED matrix

All three historical preflight roots remain intact, with their input-change
explanations in the parent SPEC:
`/tmp/worker3-centering-native-preflight-4aq04pyh`,
`/tmp/worker3-centering-native-preflight-luzd_fps`, and
`/tmp/worker3-centering-native-preflight-e7ijpo7c`.
Each round had Native9/9 exit0 in both modes. Only the last round removes both
XML-authoring ambiguities. These are old reachability witnesses, not C2 RED
on merged C1; no rerun substitutes for explaining the earlier inputs.

**First committed C2 RED case:** implicit sharing of integer id1 across two
model files with different widths. Authoritative preflight Native Xmax176.69
versus Ares179.69 is the known3mm geometry defect. Native explicit key1 and
implicit id1 had identical raw output SHA in each mode; Ares implicit matched
the distinct-mesh control instead. Preserve this input distinction and use
strict ordered full-output comparison; extent checks are supporting evidence.

| RED/control group | Failure modes to cover |
| --- | --- |
| Implicit collision first; distinct-id control | Wrong path-qualified key or independent second mesh; false sharing of distinct keys. |
| Explicit key, unseen key, `-1` loaded and unseen | Presence-only bool, wrong default/sentinel handling, failure to register a newly encountered owner. |
| Signed/prefix key controls | Strict parsing or accidental unsigned key; Native-admitted representable `atoi` cases. |
| Repeated same-path leaf, nested BFS, reversed build order | Wrong owner order, duplicate recentering, output-order changes. |
| Nonzero owner shift and different later shape; rotated/scaled components | Reused raw mesh, wrong init shift, fresh/shared multiplication order. |
| Two plates with different eligible first meshes | All-object cache contamination; wrong owner after excluding a plate. Compare independently imported Native plate outputs. |
| One loaded Project selecting A,B,A | Mutation, stale sharing or twice-normalized instance offsets; strict per-plate Native/API/CLI and repeated API byte equality. |
| Distinct current-volume settings/type and existing support annotations | Copying the owner's whole volume instead of only its mesh; leaking supported facets. Use existing admitted topology/annotation behavior. |

Combine fixtures only where all failure modes remain independently observable.
The plate-local witness and repeated-selection witness must exist before GREEN;
they cannot be inferred from one-plate results. Public API and actual CLI must
compare strictly against official and seeded Native, with immutable input SHA,
producer SHA/commands/env, effective settings and comparison artifacts.
Store only authored3MF inputs/provenance in Git; Native products stay external.

## Commit and release order after approval

1. Fixtures and public Native RED, separate test commit; report exact RED hash
   and both-mode evidence to inbox before production.
2. Mechanical assembly split in its own refactor commit, with reconstruction
   evidence and unchanged controls; no C2 behavior mixed into that commit.
3. GREEN loader/domain implementation and removal of the obsolete guard.
   Any conflicting guard-only test assertion needs its own retirement request
   citing this source boundary and the new RED; this SPEC grants no deletion.
4. CPU12 outer-taskset directed jobs2/threads2, fresh cache/artifact roots for
   every run; C2 matrix and C1 controls strict in both modes. Workspace clippy,
   fmt and wasm32 green, source audit and code REVIEW-REQUEST.
5. Only allocated workspace/full; canonical candidate official+seeded full
   requires explicit FULL-GO and fresh per-leg target/cache/root, fixed
   jobs2/threads2, zero FAIL. START/END to inbox. Exact-hash APPROVED before
   coordinator-owned merge; worker does not push.

No probe, lint suppression, ignored/skipped case, weakened assertion,
`regions[0]`, output normalization or legacy fallback is permitted.
