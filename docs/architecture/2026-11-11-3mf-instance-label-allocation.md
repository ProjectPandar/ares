# Fresh 3MF instance label allocation

## Source boundary

This slice rewrites the imported-instance allocation contribution to
`ModelInstance::get_labeled_id`, not an Ares identity scheme. References are to
OrcaSlicer revision `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `src/libslic3r/ObjectID.hpp:76–87,100–111`: ordinary `ObjectBase`
  construction increments `s_last_id`; the integer-argument deserialization
  constructor does not. `ObjectWithTimestamp` uses that ordinary constructor.
- `src/OrcaSlicer.cpp:1730,1757`, `Model.cpp:331,353–354`, and
  `Model.hpp:1593–1610,1695–1704,1753`: the outer CLI model and the model
  returned by `read_from_file` each allocate a model and its wipe tower.
- `Model.hpp:582–588`: a fresh model object allocates itself, its configuration,
  and its layer-height profile, consuming three IDs.
- `Format/bbs_3mf.cpp::_create_object_instance` (`4381–4398`): the first
  `(model path, object ID)` build item constructs the model object and then its
  first instance. The existing-object branch (`4424–4432`) adds another instance
  without another model-object allocation. Build order, not sorted resource IDs
  or volume order, determines these allocations.
- `bbs_3mf.cpp:2179,2317` and `_generate_volumes_new` (`5212` onward): imported
  volumes are generated after the build instances. Their allocations must not
  be inserted between these build-item instance allocations.
- `bbs_3mf.cpp:2466–2500` assigns a valid plate entry's `identify_id` to an
  existing instance's `loaded_id`; missing entries do not invent a loaded ID.
  `Model.hpp:1433–1443` returns a positive loaded ID when loaded labeling is
  enabled, otherwise the instance's allocated ObjectID.
- `GCode.cpp:4224–4239,4336–4340` consumes that labeled ID for the model-label
  header and label inventory. It is not the XML object ID, build instance index,
  physical extruder, or logical filament ID.

For the retained single-object BBL project with no `model_instance` entries,
these constructors give `2 + 2 + 3 + 1 = 8`. This is a consequence of the
construction sequence, not a permissible replacement constant.

## Architecture decision and Rust destination

Keep the counter local to a fresh byte-input model read. Do not port a
process-global mutable counter into platform-neutral core or add a public
identity API. Represent the constructor contributions shared by the existing
STL import and 3MF build traversal in a private
`crates/ares-core/src/project/domain/object_id.rs` module.

The existing single-STL expression in
`project/domain/profile_input/stl.rs::Project::from_stl_with_profiles` is a
compatibility shell around these upstream constructors. Replace that expression
with the shared constructor counting, preserving its volume-before-instance
sequence and the accepted label and plate identity 22. Its volume contribution
remains 14: volume, configuration, four facets, and eight texture-displacement
facets (`Model.hpp:903–935,1205–1220`; `Model.cpp:1402–1404`). No STL behavior,
profile entry, placement, or command changes belong here.

In `project/load/assemble.rs::project_domain_with_budget`, advance this counter
at the existing first-object and every build-instance events. Count a first
object once per existing `(PackagePath, object ID)` builder identity. Do not
advance it while Rust materializes volume geometry inline: the upstream 3MF
volume constructors are deferred until after the imported instances.

`project/domain.rs::ProjectInstance` and `project/domain/instance.rs` retain the
distinction between a loaded label and an allocation-derived label. Preserve
`loaded_label_id()` returning `None` for missing or zero metadata and returning
the original positive value for loaded metadata. Add only a crate-private
labeled-ID accessor for the upstream `get_labeled_id` result. Advance the
allocation sequence even for instances whose loaded label takes precedence.
Do not encode an absent allocation as a guessed zero or reuse an XML ID.

Replace the zero substitutions in `gcode_emit/object.rs::ObjectLabels` and
`gcode_emit/layers/instances.rs::collect` with this same accessor so headers,
inventory encoding, starts, and stops use one source-owned identity. Preserve
plate selection's use of the optional loaded label; allocation fallback does
not create plate metadata or change the geometric print-volume gate.

This is a bounded replacement of the import construction-count shell, not a
complete model/ObjectBase lifetime port. Core keeps no filesystem, terminal,
OpenGL, global mutable state, unsafe code, or new dependencies.

## Included and deferred behavior

Included: fresh BBL/Orca 3MF explicit-root-build import; first and repeated build
items, multiple root objects, positive loaded-label precedence, absent/zero
loaded metadata, and the existing direct-STL construction count. Component
resource meshes do not themselves create model objects during this build pass.

The currently parsed project/domain model remains a temporary compatibility
shell around `Model`/`ModelObject`/`ModelInstance`, not a claim of complete native
lifetime parity. Deferred: multiple-file imports in one CLI lifetime, GUI or
process-global allocation history, new IDs for clones/undo/deserialization,
post-import plater construction lifetimes, model materials and extra geometry
constructors not on this import path. In particular, the non-BBL/non-published
multiple-instance splitting branch (`bbs_3mf.cpp:2137–2182`) clones objects after
loading; this slice does not claim parity for that separate branch. Do not add
reconstruction, an alternate identity policy, or a legacy fallback for it.

No nozzle template, inventory, G-code schedule, variant list, profile refresh,
exporter, comparison, or output normalization change is part of A. C's three
variant-list members remain an independent source-cited slice.

## Native-first public verification

Use the existing `NativeCase::slice` then `NativeCase::compare` seam: the same
retained 3MF bytes go to Native, the public byte-oriented API, and CLI. Do not
add a private counter test or expose identity solely for testing. Preserve the
existing generator-only ordered complete-output comparator, including headers,
metadata, timing and footer, and require raw API/CLI equality.

Add a BBL input derived from the already retained H2D scalar fixture. Preserve
its printer/process/filament settings, templates, diameter inventory, meshes,
and all unchanged archive members. Change only root object/build occurrences
and model-instance metadata. Use three build instances of the first object,
then one of a second object; only the third instance has loaded label 37. The
source allocation sequence is 8, 9, 10, 14 and the expected Native labeled IDs
are 8, 9, 37, 14. This distinguishes constant8, XML/index substitution, resetting
per object, skipping loaded instances, and allocating 3MF volumes before their
upstream time. Native must establish these literal premises in both official
and seeded modes before any Ares execution or parity claim. Also qualify an
unchanged-geometry derivative with no `model_instance` entries and Native label8,
matching the retained system project's missing-metadata premise.

Failure modes to preserve before implementation: wrong constructor prefix;
volume-before-build counting; counting a reused object again; skipped allocation
when a loaded ID wins; conflating loaded/allocated/XML/index identities; losing
`None` metadata semantics or plate membership; headers and queued label inventory
using different IDs; perturbing accepted STL22; setup/Native premise failure;
API or CLI emission failure; and actual complete-output mismatch. Preserve
separate evidence and first differences; no output means no comparison claim.

Commit the spec, then tests demonstrating the current label mismatch with Native
already successful in both modes, then the source fix. Intermediate failures of
the retained primary H2D test are authorized until the independent A/C corrections
are complete. Final qualification is the frozen21 plus the added A/C tests in
both modes, fresh fmt/strict locked workspace-all-target Clippy/core+WASM/LOC,
then W2 rebase after B54 main landing and one final review request. Changed Rust
files stay at most400 lines; `runner.rs` must not grow.
