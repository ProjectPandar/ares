# Saved build-instance transforms in sliced 3MF export

## Native boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) reads the
12 coefficients of a 3MF build transform in column-major affine order
(`src/libslic3r/Format/bbs_3mf.cpp::bbs_get_transform_from_3mf_specs_string`,
637–664). `_BBS_3MF_Importer::_handle_start_item` (4118–4134) passes that
matrix and the printable flag to `_create_object_instance` (4423–4433).
The model-end callback applies the deferred transforms (3765–3769).
`_apply_transform` (4464–4472) accepts a finite invertible linear block and
assigns the matrix as a `ModelInstance` transformation. It does not discard
rotation or decompose the matrix to decide whether to apply it. The importer
can subsequently fold a single volume's nonzero offset into instance offsets
(2337–2358); a centered part with no effective volume offset does not need
that adjustment.

For selected-plate sliced export, `src/OrcaSlicer.cpp:8081–8086` passes the
loaded model to `export_project`. `_BBS_3MF_Exporter::_add_model_file_to_archive`
(7419–7427) copies `instance->get_matrix()` and its printable flag into the
build list. `_add_build_to_model_stream` and `add_transformation` (7844–7876)
serialize the matrix in the original 3MF order, using the model stream's
nine significant digits (7143–7151). This is distinct from the component
matrix and mesh resource; those serialize the local volume, not a baked
world-space instance (7501–7517, 7548–7564, 7647–7689).

## Ares destination and scope

`project/load/assemble.rs::project_domain_with_budget` already keeps a
build item's typed transform on `ProjectInstance`. On the full saved
`--slice 1 --export-3mf` route, `project/export/sliced.rs::CliSavedSlicedProject::export_inner`
reuses that admitted project for slicing and for model export. The gap is
in `project/export/saved.rs::from_saved_project_with_extras`: a combined
check rejects any nonidentity linear block and the saved-instance projection
reconstructs a translation-only `Instance::at`. This refuses a valid build
transform even if the slicer can already process the same project.

Extend that saved-instance projection only for the full sliced archive of
one saved plate, with one printable build instance per object. An object
with a newly admitted build matrix has one ordinary, centered, unpainted
part with an identity effective component transform and part source matrix.
Use the admitted instance's complete affine matrix for
`project/export/geometry.rs::Instance`. `project/export/xml.rs::main_model`
already writes that matrix and the printable flag in the required order and
precision. Preserve the input translation, local mesh, part and assembly
state, output identifiers, and existing path, archive, plate, and post-load
checks. Do not reopen the positional archive or add a new public API, a
second placement model, or a geometry bake. Slicing and result reporting
continue to use the same loaded project. Do not tie matrix admission to
whether the archive has optional saved extras.

Port the exact build-matrix predicate from `_apply_transform`: all affine
coefficients must be finite and the 3×3 linear block must be invertible under
Eigen's default `FullPivLU`. Orca pins Eigen 5.0.1 (`deps/Eigen/Eigen.cmake:6–9`)
and uses a double affine (`src/libslic3r/Point.hpp:96`). In Eigen 5.0.1,
`Eigen/src/LU/FullPivLU.h:489–539` selects the largest absolute coefficient
in each remaining corner and eliminates with row and column pivots;
`:321–342,362–387` requires all three pivot magnitudes to be **strictly**
greater than the largest pivot magnitude times `3 * f64::EPSILON`
(`Eigen/src/Core/NumTraits.h:205`). Reuse that rank rule for the fixed-size
linear block, including the pivot order; a nonzero determinant alone is
not equivalent. Do not decompose, round, or alter the accepted matrix.

That predicate admits ordinary scale (including nonuniform scale), shear,
and reflection as well as rotation. The existing slice path multiplies the
full linear block into slice vertices and flips triangle winding when the
composed determinant is negative (`project_slice/raw_intersections.rs:129–151,250–266`).
The export XML writer also accepts the full matrix. Existing coordinate and
slicing checks still apply. Rank-deficient matrices are not newly admitted:
Native skips applying them; this change does not introduce an import fallback
or change invalid-input behavior. Exact identity and translation-only cases
retain their existing behavior. Noncentered or transformed parts on newly
admitted affine instances, and multiple instances, remain outside this slice.

Restrict the new shape to the existing saved metadata that the writer can
round-trip, as in the synthetic translation-only save controls: safe object
and part names, default object extruder, default auto-drop, an explicit
portable part source filename/provenance, and no unsupported mesh repair
statistics or overrides. Do not silently make a previously refused affine
instance writable by substituting the default extruder or dropping source
provenance. This does not narrow the behavior of already-admitted
translation-only projects. Nonprintable instances and unsafe names remain
unsupported. Unsliced export, `--min-save`, the
separate processed-plate compatibility entry point, assembly input,
plain STL/3MF, explicit CLI transforms, and changes to the slicer are not
part of this slice.

## Metadata kept distinct from the build transform

All ranges below refer to `bbs_3mf.cpp`. These fields describe the associated
Native load/store seams; they do not get folded into the instance transform.

| Field | Native load → store | Scope |
| --- | --- | --- |
| Build `printable` | `4127–4134`, `4423–4433` → `7419–7427`, `7867–7876` | Keep the printable-only admission. |
| Component `transform` | `_handle_start_component` `4076–4095`, volume composition `5366–5368` → `7548–7564` | Leave local volume transforms and the existing translation-only route unchanged. |
| Part `matrix` | `_generate_volumes_new` `5250–5257,5360–5368` → `8362–8372`, volume matrix multiplied by source matrix | Do not substitute it for a build matrix. Nonidentity part source matrices with newly admitted affine instances are separate. |
| `assemble_item` `transform` and `offset` | `_handle_start_assemble_item` `5019–5047`, readers `637–684` → `8617–8647` | Retain independent initialized assembly state using the existing [assemble state implementation](3mf-assemble-instance-state.md). |
| Object `name` | `4577–4583,2272–2279` → `8315–8317`; split path uses object name and backup ID (`7341–7361`) | Keep the established safe-name and package-path handling. |
| Object `extruder` | Config deserialization `2272–2280`, filament range handling `2376–2394` → config serialization `8323–8326` | Nondefault object assignment and arbitrary object config round-trip are separate from the build transform. |
| Object/build `auto_drop` | `4131–4132,4486–4505,2263–2270` → object attribute `8312–8313` | No new auto-drop behavior. |
| Part `name` | `5425–5428` → `8350–8353` | Keep existing part-name handling. |
| Part `source_file` | `5440–5441`, ordinary-part fallback `2360–2374` → basename/full-path policy `8374–8387` | Reuse already-supported explicit portable provenance. Do not implement fallback from the input path here. |
| Part `source_object_id` / `source_volume_id` | `5442–5445`, unset-ID fallback `2360–2374` → `8380–8383` | No fallback change. |
| Part `source_offset_x` / `source_offset_y` / `source_offset_z` | `5446–5451` → `8384–8386` when source file is present | No provenance or offset changes. |
| Part `mesh_stat` | `4540–4560`, mesh construction and orientation `5330–5346` → live mesh repair statistics `8415–8422` | Do not add raw statistics passthrough or a repair rewrite. |

## Follow-ups

- **Round-trip nondefault object `extruder`.** Native deserializes object
  config (`bbs_3mf.cpp:2272–2280`), applies the filament-range/default and
  single-volume policy (`2376–2394`), then serializes object config
  (`8323–8326`). Port this as a separate saved-object metadata slice instead
  of extending the CLI-default metadata record in the matrix change.
- **Populate absent part `source_file` and retain companion provenance.**
  Native reads `source_file`, `source_object_id`, `source_volume_id` and
  `source_offset_x/y/z` (`5440–5451`); ordinary parts with no source filename
  get the input filename and only unset indices are filled (`2360–2374`).
  The writer chooses the basename unless full-path sources were requested
  and emits the source IDs and offsets with the file (`8374–8387`). Port the
  fallback and companion fields together, separately from build transforms.
- **Serialize live `mesh_stat`.** Native parses repair counters
  (`4540–4560`), uses them in mesh construction and checks orientation
  (`5330–5346`), then writes the live mesh's repair statistics (`8415–8422`).
  Trace the load-to-store mesh state and port its statistics separately;
  do not treat the source XML attributes as an opaque block to copy.

## Synthetic verification

Use the existing saved-sliced parity harness and generate an independent
single-plate saved fixture from a closed asymmetric prism with distinct axis
extents, safe synthetic names, and ordinary supported metadata. Center the
local resource and preserve an identity component and part source matrix.
Set only the build matrix and bed-safe translation. Use one exact proper
rotation that moves the Z axis onto a horizontal axis, placing a broad face
on the bed, and a non-cardinal planar rotation with coefficients derived
from a 3–4–5 triangle to exercise serialization beyond signed permutations.
Add a reflected instance with a uniform power-of-two scale on an asymmetric
mesh and a separate simple XY shear, both with bed-safe translations. The
reflected case exercises winding in slicing without changing the serialized
local triangle indices. If a case exposes a separate slicing defect, isolate
and report it rather than hiding a geometry or G-code difference. Use the
unchanged translation-only saved-sliced fixture as the control. Keep an assembly-state
or second-object case in adjacent coverage so shape-specific projection
cannot replace assembly state or reorder object records.

Invoke the same selected-plate full export in Native and Ares. Check
successful status, one archive, no standalone G-code, and the expected
build linear block and translation by object reference. Compare all members
with the existing strict sliced-archive comparator (including component and
mesh bytes, settings, slice metadata, plate statistics, G-code and its raw
checksum); do not normalize build or component matrices or add masks. The
new tests must first observe the Ares saved-model refusal after proving
Native succeeds. Verify the corrected behavior in both pinned Official and
Seeded lanes and pair adjacent saved/sliced/unsliced/STL/assembly-state,
archive extras, version admission, and CLI warning coverage against the
parent. Use only independently generated fixtures; never use production
archives or their derived data as regression fixtures.
