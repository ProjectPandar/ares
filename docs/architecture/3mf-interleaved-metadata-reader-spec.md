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
