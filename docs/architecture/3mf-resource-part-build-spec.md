# SPEC: 3MF resource-part import without a build element

Status: proposed (revised input pair after the first RED attempt, whose control
failed full parity). Old test/fixtures remain immutable.

## 1. Owning upstream rewrite boundary

Pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`, paths below relative OrcaSlicer/src.

`libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer::ObjectImporter` is the owner:
- `_load_model_from_file:1740-1829` loads relationships, then **active** ObjectImporter
  workers1780-1817 and finally the main model1826-1829. The earlier sequential
  submodel `_extract_model_from_archive` loop is inside `#if 0`; do not cite it
  as the active resource parser.
- `ObjectImporter::extract_object_model:993-1021` selects object_path from ZIP;
  `_extract_object_from_archive:5715-5779` streams it through the object parser,
  preserving XML parse errors, resource objects and their part identity.
- `_handle_object_start_model:5316-5321` captures units; end_model5323-5327 does
  not require build. Start/end dispatch5612-5687 handles resources, objects,
  meshes, components and metadata, **not build/item**. Resource import therefore
  does not require a build tag or create main-model instances from its absence.
- Main parser dispatch3350-3435 is distinct; `_handle_start_build/_end_build:
  3881-3891` are no-ops, while `_handle_start_item:3893-3909` creates instances
  via `_create_object_instance`. This slice does not change main-model policy.

This is a source-cited port of the resource ObjectImporter/main-model distinction,
not an Ares-designed ZIP/scene pipeline or a general 3MF-conformance project.

## 2. Retained finding, and what it does not establish

Frozen `loader-no-build-3mf-finding.md`.
Exact v1 contour.3mf SHA
`871b1a88ee8813d468ee65d6a2a6b7e22097788d43d06c2e8a61f19481dce880`.
Its main `3D/3dmodel.model` HAS build. Body
`3D/Objects/ring.stl_1.model` and modifier resource XML lack build.

Historical base48ff6b184: official Native exit0/complete G-code, SHA
`a5f71d5321755deb70164ca6b859994dafeefc9e0a7a3a80ae2fdb7b9803421f`;
public slice_with_profiles failed `InvalidInput("invalid project model XML: missing
field `build`")`. Historical Native received flattened machine/process/filament
JSONs; the API received catalog/selection/overrides. All supplied sources/argv
remain in the archive, not inferred from preset IDs. This is not evidence for the
new bare-3MF canonical slicing request contract below.
Seeded0, admitted ordered-G-code comparisons0. This was an
admission finding, NOT a lower-slice/seam/ordered-stream RED. It is not a fresh
public loader RED on current main or proof of complete model parity.

Retained v2 added only empty build elements to resource parts. That fixture
compatibility correction did not repair the loader and must not replace v1 to
make the positive pass. Before/after ZIP-entry mesh/options/transform proof is
retained. This proposal does not resume the old eight-case discovery or seam v2.

## 2a. Rejected first public attempt and replacement input justification

Frozen test e1de695aa8cfbffb99cd971f86468037f6df4766 is **NOT a qualified RED**.
Fresh Native-first official+seeded6/6 complete/neutral; official public positive/
repeat API and CLI both reject missing build. However old v2 control admitted/
raw API==CLI but strict comparator failed byte5217/line224: M73 P67 R2 versus
adjacent G1 order. Public seeded was not run.
[Independent discovery](discovery-m73-resource-control-order.md) records the
GCodeProcessor::run_post_process insertion boundary; it is deferred, not repaired
by this slice. No comparator/M73 weakening or retry of the old pair is proposed.

**New control:** byte-exact current-main fixture
`tests/parity/wipe-before-override/control-before50.3mf`, SHA
`adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2`.
It contains a valid main build and one referenced mesh resource with empty build;
no modifier is required to witness the same upstream resource parser boundary.
This replaces geometry AND settings as a whole with an existing qualified input,
not selectively disabling M73 or editing values to hide its difference.

**New positive:** remove ONLY the literal `<build/>` from
`3D/Objects/overhang.stl_1.model` of that control, SHA
`0eab5d4dea27aa756422e4a821d8d64d1b2ac544afc83dd121ac4899afa33f7a`.
All other uncompressed ZIP entries, order, resource vertices/triangles, main build,
relationships, volume IDs/roles/transforms and embedded settings remain identical.
The resource entry hashes are control
`5de32760cab07a393137fa0e6d485b83520352650fea421f6c78156c5e6cf78b`
versus positive
`345b6dc35a34402a1a6e06733b4829dfa04acb8fedc407b4e831d509de7c4d2e`.
Only this positive is derived; control is an exact existing file, not regenerated.

**Current-main strict-PASS evidence, not a new runtime claim:** the full run's
`wipe_before_override::wipe_before_override_control_before50` passes
in official and seeded modes.
Raw retained streams were read offline: strict generator-only COMPLETE equality
and raw API/CLI equality independently verified, retaining M73/all movement/footer.
Both API+CLI rawSHA
`2e9e3d4a65a71c9e05b58792c78e3b21879b635ab219be0b212d39da7f58340d`;
all normalized Native/API/CLI SHA
`8bae941305727f043ec25c0cca7a7bd8d0042d12cdc597b6ac1ed6b214567f69`.
Actual source request was Native --slice0 --outputdir (no explicit --arrange),
validated API/defaults and real bare CLI. Preserve that exact source request in
this revision; no arrangement option or loadedJSONs are added. Fresh Native-first
proof on the new pair is still mandatory. No Native/
API/CLI execution has been done on the new positive and no positive RED claimed.

## 3. Exact Rust destination / behavior

`crates/ares-core/src/project/load.rs::load_project_with_preparation:87-96`
resolves the main model from relationships, then calls `load/graph.rs::load`.
That loader:94-139 deserializes EVERY queued part as `model_xml::ModelDocument`.
`project/model_xml.rs::ModelDocument:13-23` requires build for all parts; Build:
210-214 permits empty item lists only once the element exists. This transport
scaffold conflates upstream main-model and resource ObjectImporter roles.

**Replacement boundary:** private part decoding at
`project/load/graph.rs::load` and `project/model_xml.rs` (or a small private
resource-decoding sibling if needed for <400LOC). Use the already resolved main
PackagePath to distinguish referenced resource parts. Reuse existing archive,
namespace/security checks, content types, relationships, mesh validation and
identity graph. Do not infer role from filenames, catch errors and reparse, patch
XML strings, synthesize root instances, or add an independent importer/API.

Included: referenced resource models with valid existing resources/components,
with build absent or empty, yield identical loaded resources and component
identities. Main build still owns instance assembly
`load/assemble.rs::project_domain_with_budget:108-128`; resource omission must
neither remove body/modifier meshes nor instantiate extra objects.

Main-model missing/empty build semantics are **not established here**. Preserve
current main-model behavior; do not silently broaden it with a global serde
build default. If preserving it requires a different boundary or reveals an
upstream discrepancy, STOP/amend the SPEC rather than invent Unsupported guards.
Existing XML/schema/limit failures and path/identity checks stay unchanged.

Existing ModelDocument/ModelGraph remain temporary representations of the named
upstream resource/import graph, not a new Ares scene architecture.
Tier1 byte-only core boundary stays intact.

## 4. Public RED seam: Native-first canonical slicing

**No loader-only or export-only oracle.** The earlier load_project/export proposal
is withdrawn. The owning production boundary remains resource-part parsing, but
verification uses the complete public slicing path:

- Native `--slice 0 --outputdir <fresh>` on the identical frozen3MF, matching
  the qualified current-main control request without adding --arrange;
  no load-settings/filament JSONs or extra overrides for this bare-3MF contract.
- Public `ares_core::slice_project_plate_validated` on those same bytes, no profile,
  plateNone/default ConfigOverrides, matching deterministic generation metadata
  and SlicerContext seed0.
- Real CLI `ares slice -o <fresh cli.gcode> <same frozen3MF>`, no profile/loaded
  files/overrides. Capture actual command/status/stdout/stderr and complete output.

Freeze the new section2a pair: positive0eab5d4d, exact qualified
controladb38ee8, and fresh positive repeat. Pin complete source/ZIP-entry hashes
and prove the sole resource empty-build removal. No further mesh/settings/
M73/config/metadata/identity/transform changes; no old-v1/v2 substitution.
Retained full-run control streams select the fixture, but fresh Native references
are mandatory for this RED matrix.

**Native-first, full test per mode:** obtain fresh Native --slice references
before public calls within EACH official/seeded test invocation.
No Native-only early return or externally injected reference path is permitted;
each enabled test always executes Native→public end to end. Every reference leg must
exit0 with complete ordered G-code/footer and authoritative complete snapshots.
Require complete generator-only Native positive/repeat/control neutrality; no
cached historical stream, export result, partial observation or load-only success
can substitute. Stop on Native failure, unstable/non-neutral reference or source
mismatch. Do not run Ares first and treat a missing Native baseline as RED.

**Expected RED:** Native completes G-code on the frozen positive while the public
Ares slicing entry rejects its resource model specifically for missing build.
Run/capture the real CLI too: it must reject the same request for the same
admission cause, not a different adapter/input contract. This attributable product
admission error is the RED at the slicing seam; it is not fabricated
ordered-byte divergence. Native/harness failures or unrelated Ares errors are not
qualifying RED. Preserve the old discovery's separate historical classification.

For every admitted Ares leg, require API/CLI RAW byte equality and the unchanged
strict COMPLETE ordered Native/API/CLI generator-only comparison: no removal of
motion/arcs/M73/statistics/footer, no effective-config-equality admission gate,
no tolerance or metadata filtering. Save full configs and first difference.
For an admission-error leg, raw-stream equality/comparison is explicitly **not
evaluated: no complete Ares stream**; matching API/CLI error causes are recorded,
not misreported as byte equality or G-code PASS. Control must Native-complete
and both public Ares entries admit; record its full strict comparison. A residual
control mismatch is retained/reported, never waived or repaired to manufacture
parity. Freeze test-only errors/results before production.

After GREEN, positive/repeat/control must admit, API/CLI raw
equality must hold and the same full strict comparator must be rerun. If ANY
residual G-code difference remains, **STOP and report exact first difference and
full retained streams; no scope growth** into seam, geometry, order, config or
travel. Do not label loader admission as complete slicing GREEN, weaken the
comparator, substitute fixtures or repair adjacent owners.

Artifacts: exact request/source hashes, Native and CLI commands/status/snapshots,
full streams where present, public API admission error or output, configs,
repeat/control neutrality and unmodified comparator results. No committed Native
outputs, diagnostic core I/O, new public APIs or oracle fallback.

## 5. Gates / limits / deferrals

1. Freeze the new positive / repeat / exact qualified empty-build control and
   run Native-first, maximum three fresh Native slices per mode, official+seeded,
   fresh caches/artifacts, from_env. No export preflight or additional oracle.
2. Freeze public test-only attributable slicing-admission RED +
   control/neutrality proof before production.
3. Bounded correction: fmt/strict clippy/WASM,
   <400LOC modules, original slicing witness + existing project-loader family.
   Any post-GREEN residual whole-output diff means STOP/report, not expansion.

STOP on changed failure, no RED, non-neutral control, request mismatch, Native
--slice failure/incomplete G-code or missing archived input/source.
No retries or further substitutions.

Deferred: main-model missing-build parity, nonempty resource build/item behavior,
arbitrary resource-schema relaxation, nested/remote relationship-policy changes,
unit/transform/instance semantics changes, mesh repair/color/paint rewrite,
config precedence, lower-slice geometry, seam/travel/perimeter fixes and new
Native diagnostic hooks. No fallback, invented guard, fixture substitution or
Ares-owned loader/pipeline design.
