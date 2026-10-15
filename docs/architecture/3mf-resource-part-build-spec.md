# SPEC: 3MF resource-part import without a build element

Status: **docs-only REVIEW-REQUEST**. FULL16 freeze remains active. This proposes
an independent loader slice, not implementation, fixture, Native or nextest budget.
Approved inherits_group RED6f1f48a29/GREENed6bfd3aa remain unchanged, queued stack16.

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

Frozen `loader-no-build-3mf-finding.md` and archive
`/home/indexyz/.ares-evidence/worker4/lower-slices-admission-stop-CXWg.tar.gz`, SHA
`3d575de80a7ebaf18350119bb19eba06f0dc29974365cdce247e51d01d3f8c42`.
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
new standalone export/loader request contract below.
Seeded0, admitted ordered-G-code comparisons0. This was an
admission finding, NOT a lower-slice/seam/ordered-stream RED. It is not a fresh
public loader RED on current main or proof of complete model parity.

Retained v2 added only empty build elements to resource parts. That fixture
compatibility correction did not repair the loader and must not replace v1 to
make the positive pass. Before/after ZIP-entry mesh/options/transform proof is
retained. This proposal does not resume the old eight-case discovery or seam v2.

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
upstream resource/import graph, not a new Ares scene architecture. No production
change is authorized by this document. Tier1 byte-only core boundary stays intact.

## 4. Proposed public RED seam (separate approval/budget required)

Agree the seam before test code: public `ares_core::load_project(bytes)` admission
and existing public Project model/object/volume/mesh/instance views. The assertion
under test is **resource-part admission and data preservation**, not ordered-G-code
parity. At this seam a specifically attributable missing-build error can be a
product loader RED; arbitrary harness/Native admission failures cannot.

Propose exact retained v1 as positive, byte-audited v2 empty-build counterpart as
control, plus fresh positive repeat. Do not change meshes, transforms, roles,
profiles, metadata, main build, relationship paths or IDs. Freeze all entry hashes
and a proof that only the two resource empty-build tags differ before execution.
No newly modified fixture is implicitly approved by the old discovery budget.

Proposed Native admission witness is the existing public **export-only** route:
`OrcaSlicer.cpp::CLI::run:5618-5620,7058-7063` (--export-3mf), arrange0, same
standalone 3MF bytes, no load-settings/filament files or slicing request. It must
finish exit0 with a valid model export for both inputs. Native-export resource/
instance fingerprints must be positive/repeat/control invariant; exclude only
non-model container identities such as ZIP times from that *model-data* artifact,
not vertices, winding, IDs, transforms, volume roles or ordering. Export itself
is not proof of all model fields: retain source ZIP entry proof and raw exports.
This export route is proposed, NOT preflighted in this task. If unavailable or
non-neutral, STOP for review; no automatic slice/observer/oracle substitution.

Public API control must load successfully; positive must fail specifically at
resource build decoding before production. After a separately approved fix,
positive/repeat/control must load and produce identical ordered public-domain
fingerprints (mesh coordinates/index buffers, resources, roles, instances and
transforms). Artifact includes exact request/input hashes, Native command/status/
raw export, public error or fingerprint, and control/neutrality results. No
committed Native outputs, diagnostic core I/O or new public APIs.

No G-code is required by this loader-only seam. If any slicing is later proposed,
request a separate matched-input budget; retain full existing generator-only
ordered comparison, motion/arcs/M73/statistics/footer and raw API/CLI equality.
Do not use known downstream divergence to reject a valid loader finding or claim
that loader admission establishes slicing parity.

## 5. Gates / limits / deferrals

1. Exact docs SPEC/ARD approval and oracle/request-contract ruling.
2. Explicit fixture and post-freeze preflight/RED budget: proposed maximum three
   export admissions per mode, preflight included (positive/repeat/control), official+seeded, CPU15,
   jobs1/threads1, load<60, fresh caches/artifacts, from_env, START/END receipts.
3. Freeze public test-only attributable loader RED + control/neutrality proof;
   independent exact-hash review BEFORE production approval.
4. Separate bounded correction; fmt/strict clippy/WASM, <400LOC modules, original
   admission suite + existing project-loader family. Coordinator schedules full.

STOP on changed failure, no RED, non-neutral control, request mismatch, Native
admission/export failure, missing archived input/source, or freeze. No retries.
FULL16 freeze currently prohibits ALL new Native/nextest, including preflight.

Deferred: main-model missing-build parity, nonempty resource build/item behavior,
arbitrary resource-schema relaxation, nested/remote relationship-policy changes,
unit/transform/instance semantics changes, mesh repair/color/paint rewrite,
config precedence, lower-slice geometry, seam/travel/perimeter fixes and new
Native diagnostic hooks. No fallback, invented guard, fixture substitution or
Ares-owned loader/pipeline design.
