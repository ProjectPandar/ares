# SPEC: shared 3MF triangle vertex-index zero defaults

Status: **APPROVED a1c1b81dac38fba08e170bc75ab3b8724755677f; public RED authorized.**
Public RED freezes an input-only pair under `tests/parity/3mf-triangle-index-defaults`:
positive SHA `a19533ceee3dea510848bf4ee578a3b48f5266139c79fd288ea15da546c8e20d`,
control SHA `adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2`.
The test generates Native references at runtime; no production change authorized.
Base main `41f2c90f765d42c6620583121296bb61a2cb839c`, atop approved docs refresh
`f82a77b6d1282e86eafd928d565533f09d8badae`. This is the triangle behavior
explicitly deferred by the shared vertex-coordinate SPEC, not the already
merged resource build/item ignore slice. The approved SPEC was docs-only;
this RED adds the frozen pair/public test only. No production change here.
M73 remains deferred; full/workspace runs remain coordinator-owned.

## 1. Exact source owner and Rust destination

Orca pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Format/bbs_3mf.cpp`:

- `bbs_get_attribute_value_int:508-514` initializes `value = 0`; absent attribute
  returns that zero without parsing a string. Present strings use
  `boost::spirit::qi::int_`, **not atoi**: `abc` leaves zero on parse failure,
  `5x` parses prefix 5, and `-1` parses negative then is rejected downstream at
  `_generate_volumes_new:4991-4993`. Malformed/negative/overflow behavior remains
  deferred; this slice only defaults absent attributes.
- `_BBS_3MF_Importer::_handle_start_triangle:3804-3830`, especially `3813-3819`,
  appends `(v1,v2,v3)` from that helper. Comment: missing values equal ZERO.
- `ObjectImporter::_handle_object_start_triangle:5516-5542`, especially
  `5526-5532`, does the same for resource models. Main and resource dispatch
  share this contract; there is no upstream reason for a role-specific default.

Destination: the **single shared** `crates/ares-core/src/project/model_xml.rs::
Triangle` decoder reached by `project/load/graph.rs::load` for main and resource
documents. All three `#[serde(rename = "@v1"/"@v2"/"@v3")]` fields are currently
required `u32`, with no default. Missing one fails XML deserialization before
`ModelDocument::validate`/`Mesh::validate`, at the canonical public slicing seam.
The anticipated error is a missing `@v1`, `@v2`, or `@v3` field; no new runtime
observation is claimed by this static analysis.

The bounded future port is field-level `0u32` defaults only on these three
fields. Keep present-index parsing/range checks, vertex-coordinate behavior,
paint annotations and all other fields unchanged. No blanket struct default,
role DTO split, fallback/reparse, new resolver/API, file I/O, or Ares pipeline.
Existing DTOs are transport shells around the cited upstream triangle handlers.

## 2. What Native does with index zero downstream

Index zero names the **first vertex**, not an instruction to remove a face.
It is valid if the vertex table is nonempty, and need not make a triangle
degenerate. The other two indices and vertex positions determine that.

The active `_generate_volumes_new:4894` call sites at `:2036` and `:2167` lead
to `:4975-5007`: reject an empty triangle list, copy face indices, reject any
index `<0` or `>= vertices.size()`, then construct `TriangleMesh` from the
indexed set. A zero produced by omission passes this range check if vertex0
exists. `:5020-5023` adjusts negative volume orientation and adds the volume.
The older `_generate_volumes:5131` text starts inside a block comment at `:5130`;
it is not the active importer and must not be cited as the execution path.

`TriangleMesh.cpp::TriangleMesh(indexed_triangle_set&&, ...):71-75` transfers
indices and calls `fill_initial_stats:45-54` (volume/bounds/neighbors/open edges),
not `trianglemesh_repair_on_import`. The latter at `:79-180` is used from
`from_stl:182-185`; do **not** assume STL/admesh repair is automatically run by
this 3MF constructor.

Repeated indices (e.g. `(0,0,2)`) and zero-area geometry are a separate concern:
`TriangleMesh.cpp::its_remove_degenerate_faces:742-755` exists and recognizes
repeated indices, but the active import path above does not call it.
`TriangleMeshSlicer.cpp::FaceOrientation:574-584` explicitly distinguishes
degenerate normals and notes connectivity can justify slicing such triangles.
Therefore this SPEC does not promise that Native repairs/removes every
degenerate index-0 face. Runtime admission/output of a degenerating edit would
need a separate bounded witness and scope approval.

Ares `model_xml.rs::Mesh::validate` checks finite vertices/unit-scaled values
and index `< vertex_count`; it does not reject repeated-index faces there.
Its early return accepts an empty vertex or triangle vector at this local
validation stage; do not infer whole public empty-mesh admission from it.
`load/mesh_prepare.rs::prepare:13-69` copies indices, handles signed-volume
orientation, centering and annotations. Its signed-volume calculation `:73-96`
handles zero norm without dividing by it; this is not a general mesh repair
pass. Neither path should be changed to hide a triangle default defect.

## 3. Approved pair/discovery budget (original plan; pair now frozen above)

Control: unchanged `tests/parity/wipe-before-override/control-before50.3mf`, SHA
`adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2`.
Static offline inspection of `3D/Objects/overhang.stl_1.model` finds first face
`<triangle v1="0" v2="1" v3="2"/>`. Proposed positive removes **only** its
` v1="0"` attribute. Native's zero default recreates exactly `(0,1,2)`, so this
does not create a new repeated-index/degenerate face or require a repair port.
All mesh vertices, face ordering/annotations/IDs, main build, other ZIP entries,
settings/M73/transforms remain identical. No loaded JSONs or extra overrides.

Only this v1 omission pair is proposed initially: positive + fresh repeat +
control in official and seeded, **three Native slices per mode**, CPU15. The
existing input has no explicit v2=0/v3=0 face; do not silently rotate/reindex
triangles or substitute geometry just to create extra cases. v2/v3 default
scope is source-backed by the same shared decoder and helper; separate fixtures
or main-role mesh rewrites require another input approval. A resource-only
positive exercises the decoder also used by main; no role partition is needed.

After SPEC/input approval, freeze deterministic derivation and full archive/
entry SHA manifest externally before runtime, and before any test commit. No
positive bytes or SHA were claimed at SPEC stage; the RED freeze above records
both identities. Native `--slice 0 --outputdir <fresh>`
on the same bare input, canonical official/seeded environments, fresh caches/
TMPDIR/artifacts, CPU15/jobs1/threads1/load<60/START-END. Require complete exit0,
authoritative output/footer, positive repeat determinism and expected control
neutrality; no cached/export/load-only oracle or retry/input substitution.

The expected Native pair is neutral because zero was already explicit. This is
**not** NO-RED by itself: the decisive comparison is Native versus Ares on the
**same** input. If Native completes and Ares still rejects the missing field,
that is an admission delta, despite pair neutrality. If same-input public
behavior and whole outputs are equal, NO-RED STOP. Seek adjudication rather
than repeat the former positive/control-neutrality interpretation error.

## 4. Public RED/control and GREEN gates (separate authorization)

Discovery first, then separate public test authorization. Use
`OrcaRunner::from_env()` else-return and existing reference-enabled gate; enabled
tests always produce fresh Native at runtime and perform public validated
`slice_project_plate_validated` + a real bare CLI. No Native-only PASS path,
injected reference file or cached historical output.

Control first: assert API success, CLI success, raw API==CLI and entire unchanged
strict Native/API/CLI generator-only equality. On positive, execute both public
calls before target assertions; require those same successful equality results.
At pre-GREEN HEAD a missing-field rejection must make the public test **FAIL**;
no `is_err`, `assert_ne`, expected-error or `divergent:true` passing branch.
Missing Ares output means comparison NOT EVALUATED, not missing-output equality.
Record both public calls/error receipts before the first failing assertion.

Use the approved deterministic metadata/default overrides/plateNone/context0
contract; no hidden loaded settings. Retain M73, config, ordered motion/arcs,
statistics/footer. RED review precedes production authorization, frozen tests
remain unchanged for GREEN. GREEN must make positive/repeat/control whole strict
PASS with raw API==CLI in both modes, control unchanged; targeted 3MF/load
regressions, fmt/strict clippy/WASM and byte-only Tier1 boundary preserved.
ANY residual G-code difference => STOP/report, no geometry/repair/M73 scope growth.

## 5. Failure inventory and included/deferred boundary

| Failure | Classification / required action |
|---|---|
| Frozen input/entry SHA drift or edit of nonzero index | STOP; not the approved semantics-preserving pair. |
| Native nonzero, timeout, incomplete footer/snapshot | No valid oracle, NO-RED STOP; preserve receipts, no retry. |
| Native repeat differs or explicit-zero/omitted-zero pair changes output | STOP/source adjudication; no normalization of behavior or mesh-repair expansion. |
| Native accepts, Ares API/CLI rejects missing index | Admission delta; public target-success test must FAIL until GREEN. |
| Same-input Native/API/CLI full equality | NO-RED STOP; no invented guard or forced failure. |
| API/CLI raw disagreement or different input/settings | Harness/admission mismatch; STOP before qualifying RED. |
| Control fails any admission/raw/strict assertion | Disqualify RED; no comparator waiver or downstream fix. |
| Zero has no target in an empty vertex table | Invalid initial witness; Native active volume bounds reject it, while Ares local Mesh::validate has an empty-input early return. Public empty-mesh policy is deferred. |
| Missing nonzero index creates repeated/zero-area face | Outside initial pair; do not assume automatic Native repair. |
| Annotation ordering/count or geometry changes | STOP, preserve complete artifacts; no repair to get parity. |
| Main behavior requires a separate field decoder | Reject self-designed split; both roles share cited handlers. |

Included: absent v1/v2/v3 become zero in the shared Triangle decoder; initial
public witness removes an already-zero v1 only, with all downstream checks and
geometry unchanged. Deferred: malformed/negative/overflow numeric conversion,
nonzero-index omissions, degenerate-face removal/repair, empty meshes, triangle
annotations, main fixture transport rewrites, transforms/units, component graph,
vertex coordinates (already merged), configuration and M73. No new pipeline.
[ARD](ard-3mf-triangle-index-defaults.md).
