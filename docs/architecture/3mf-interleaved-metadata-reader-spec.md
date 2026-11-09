# Order-independent 3MF metadata readers

## Source boundary

This is a Rust rewrite correction for upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`, not a new project-import pipeline.

`libslic3r/Format/bbs_3mf.cpp` dispatches model child events by element name
at3582-3593 and3634-3635. `_BBS_3MF_Importer::_handle_start_metadata`
at4142-4152 resets the current text/name; `_handle_end_metadata` at4160
interprets that element when it closes. Neither handler requires metadata
siblings to occupy one uninterrupted run before resources or build.
`libslic3r/Format/3mf.cpp:1511-1512,1549-1550,1877-1930` provides the
corresponding generic3MF event-dispatch/metadata boundary.

For `Metadata/model_settings.config`, `bbs_3mf.cpp:3655-3673` dispatches
object, part, mesh statistics, metadata and plate start events separately.
`_handle_start_config_metadata:4577-4626` routes each key/value to the
currently active object, part or plate and appends object/part entries.
Metadata eligibility depends on parent state, not adjacency to other metadata.

The Rust destination is the existing byte-input project reader:
`project::model_xml::ModelDocument`, `project::upward_machine::ModelIdentity`,
`project::model_settings::ObjectSettingsWire`, and
`project::model_settings::part_metadata::PartSettingsWire`.
These serde structures remain compatibility representations of the cited
upstream importer state. Replace their contiguous-list assumption only.

## Required behavior

A model may contain metadata before and after its resources/build children.
An object configuration may contain metadata before and after parts, and a
part configuration may contain metadata before and after mesh statistics.
Each metadata element must reach its existing consumer in document order.

Repeated XML metadata elements are a sequence, not repeated singleton fields.
Do not confuse repeated element names with repeated metadata keys. Preserve
existing ordered key handling, including existing last-value-wins behavior
where implemented; do not sort, deduplicate or overwrite entries during parsing.

Keep text-valued model metadata distinct from key/value-attribute configuration
metadata. Preserve existing entity decoding and attribute interpretation.
Do not rewrite, reorder, sanitize or copy the entire input XML to make it fit
the old wire shape. Parse the accepted order directly.

Keep required model resources, required main-model build and optional/ignored
component-model build behavior unchanged. In particular, do not deserialize
ignored component-model build contents as ordinary build items.
Retain rejection of duplicate singleton resources/build/mesh-stat fields where
those fields currently reject duplicates. Keep current attribute defaults,
missing-field errors and typed geometry/configuration decoding unchanged.

`load_project` and the lightweight upward-machine identity reader must both
accept the model-level ordering. Fixing only one leaves the other public path
broken. The WASM scene importer delegates to `load_project`; it must receive
the same core behavior, not a separate browser XML parser or UI workaround.

## Chosen reader design

Use scoped ordered-child deserialization, following the existing
`PlateSettingsWire` child-dispatch pattern. Represent the affected parents'
children in document order and fold metadata and structural children into the
existing destination fields. An ordered `$value` child wire is suitable;
keep any main/resource build-mode specialization private to the model reader.

Identity-only reads must skip resource/build contents with ignored-value
semantics rather than constructing mesh vertices, triangles or project state.
Full model reads must move the decoded resources/build into their existing
fields without a second geometry allocation or clone.

Keep the configured namespace-aware reader and shared XML validation before
deserialization. Document/depth/attribute/text bounds, entity/DTD rejection,
namespace meanings and archive/package validation are not relaxed or bypassed.
Unknown-element behavior remains whatever the existing validated reader permits.

Do not enable `quick-xml/overlapped-lists` globally for this correction.
That feature can solve list adjacency, but its skipped-event replay buffer can
retain every event in intervening resources, including large geometry.
A small edit to Cargo features would therefore change all XML readers and
could add geometry-sized event storage to identity-only and browser imports.
An arbitrary replay-event cap would instead reject otherwise valid large
projects based on their ordering. Neither change is needed for this boundary.

## Upstream face-property attribute admission

At the same upstream pin, `bbs_3mf.cpp:367` names the unqualified
`face_property` triangle attribute. `bbs_get_attribute_value_charptr:525-535`
compares exact attribute names; `bbs_get_attribute_value_string:538-541`
returns its string, or the empty string when absent. The main triangle reader
at4023-4052 and `ObjectImporter` triangle reader at5927 append that string.
Thus an unbound `triangle/@face_property` is an upstream-supported form,
not an attribute whose namespace must be the production extension namespace.

Include that exact unbound attribute in the existing model-attribute namespace
gate. Do not admit arbitrary unknown attributes, add namespace aliases, or
relax the required namespaces of existing core/production/XML attributes.
Do not preprocess or strip the attribute to bypass validation.

A runtime stock-project variant with an explicitly empty `face_property`
must import and slice identically to its absent-attribute control: the pinned
getter returns the same empty string for both inputs. Keep the public project,
identity and CLI paths covered by the same complete ordered-output contract.

Face properties can change Native output through auto-orientation. The importer
materializes them at5324-5327 and5542-5545; `deps_src/admesh/stl.h:171-218`
parses the normal/exterior-appearance type and area. `Orient.cpp:209-227`
reads the appearance type and427-428 changes the orientation's overhang cost;
selection at185-202 can consequently choose a different rotation and slice.

The named deferred gap is **FaceProperty-aware auto-orientation, owner
Orient.cpp:185-227/427-428 + admesh stl.h:171-218**. Whoever later ports
orientation must consume the preserved `FaceProperty` data. The CLI
orientation action is explicit (`OrcaSlicer.cpp:4973ff`). The current project
slice contract retains the saved transforms and does not request that action.
Prove complete Native/API/CLI equality for both a neutral attribute and an
enum-defined exterior-appearance value on this fixed-transform path. Do not
infer auto-orientation parity from that result. If an output-affecting property
is encountered on an unsupported operation, that operation must be ported or
rejected before output, not silently ignored.

Preserve the decoded strings opaquely, in triangle order, from
`project/model_xml.rs::Triangle` through `project/load/mesh_prepare.rs` into
`project/domain/mesh.rs::MeshGeometry`, shared with the existing mesh geometry.
Expose the strings through `ProjectMesh::face_properties` so the public E2E can
verify non-neutral data retention without implementing orientation. Empty
strings retain the upstream absent-attribute meaning. This requires per-face
string storage; identity-only scans must still ignore the geometry subtree.

This addition ports attribute admission and opaque retention, not the deferred
orientation algorithm. The generic3MF triangle reader (`3mf.cpp:1776-1796`) does not consume
this BBS attribute. Existing Ares mesh representations remain a temporary
compatibility shell for the deferred upstream data boundary. Namespace/entity/
archive/geometry validation stays in force; prefixed face-property attributes
and face-property attributes on other elements retain their rejection behavior.

## Keyless object metadata

`bbs_3mf.cpp::_handle_start_config_metadata:4577-4584` obtains the `key`
and `value` strings using the getter at525-541, then returns success immediately
when the key is empty. Thus a missing or explicitly empty key is skipped before
object lookup or metadata append at4609-4624. An object child such as
`metadata/@face_count` without `@key` has no imported configuration meaning;
it is not a reason to synthesize an empty option or reject the project.

Port that rule only for `object/metadata`, in
`project/model_settings.rs::ObjectSettingsWire`. Use an object-specific wire
representation that can distinguish keyless entries and discard them before
calling the existing ordered metadata consumer. Do not add serde defaults to
the shared metadata type or change part/plate metadata admission. Nonempty
keyed entries retain the current representation requirements and ordering.
Adjacent missing-value behavior for nonempty keys is not included in this slice.

Runtime stock variants must add a missing-key and an explicitly empty-key
object metadata child, using only public fixture-derived data. Native must
accept them and retain complete control output; public import/identity and
Native/API/CLI comparisons must pass. Retain the existing missing-key rejection
for part and plate metadata. No user bytes or generated archive is committed.

## Portability and allocation contract

Keep the existing quick-xml dependency/features. Use the existing Rust reader,
serde and owned project data; introduce no filesystem, terminal, UI, native
parser dependency or platform-specific behavior into core.

Preserve streaming child dispatch with respect to mesh events. Additional
ordered-child bookkeeping may scale with metadata/part children, but must not
cache and replay all skipped vertex/triangle events. XML validation plus typed
decoding remains linear in input size, without sorting or quadratic list scans.
The lightweight identity path must not gain geometry-sized allocations.
These constraints apply equally to browser WASM and native platforms.

## Included and deferred behavior

Included: interleaved metadata in main/resource model documents, lightweight
model identity, object configuration and part configuration; preservation of
existing plate metadata behavior and existing ordered semantic consumers.

Deferred: new metadata key meanings, new geometry/configuration semantics,
unrelated repeated-element lists, schema extensions, archive-limit changes,
format repair, importer pipeline redesign and upstream UI/project-database work.
No legacy parser fallback or byte-level preprocessing path is introduced.

## Observable contract

An independently produced complete Native project with unchanged metadata
payloads must retain the same project/identity meaning when only one metadata
element is moved across the relevant structural child. Model, object and part
order variants must import through the public project API and identity path;
the corresponding scene adapter must share the successful project result.
The contiguous control must remain accepted.

Malformed namespaces/entities, missing required resources/build, duplicate
singletons, wrong metadata representations and invalid geometry must retain
the existing rejection behavior. This change removes false duplicate-list
errors, not structural or semantic validation.
