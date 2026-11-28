# Live mesh repair statistics in saved sliced 3MF export

## Native boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)
loads the five `mesh_stat` attributes into the part's `RepairedMeshErrors`
(`src/libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer::_handle_start_config_volume_mesh`,
4540–4560). For a fresh mesh, `_generate_volumes_new` constructs
`TriangleMesh` with those errors, checks signed volume, and flips winding
if it is negative (`bbs_3mf.cpp:5330–5346`). The constructor assigns
`m_stats.repaired_errors` before `fill_initial_stats`, which computes other
mesh metrics without replacing the assigned errors
(`src/libslic3r/TriangleMesh.cpp:61–69,87–91,1479–1497`).
`TriangleMesh::flip_triangles` swaps the second and third indices and
negates the stored volume, but does not increment `facets_reversed` or
any other counter (`TriangleMesh.cpp:246–251,377–381,752–756`). This is
winding normalization, not an ADMesh repair pass.

If the importer finds a shared mesh, the later volume uses the first
loaded mesh instead of constructing one with its own counters
(`bbs_3mf.cpp:5271–5291,5348–5357`;
`src/libslic3r/Model.cpp::ModelObject::add_volume_with_shared_mesh`,
1443–1454). The selected-plate CLI passes the loaded model to export
(`src/OrcaSlicer.cpp:8081–8086`).
`_BBS_3MF_Exporter::_add_model_config_file_to_archive` gets the counters
from `volume->mesh().stats().repaired_errors` and emits all five attributes
in `Metadata/model_settings.config` (`bbs_3mf.cpp:8415–8422`). It does not
consult the part's original XML when writing the archive.

## Observed gap

A synthetic one-part prism on the existing saved export path exercised
`--slice 1 --export-3mf` with zero or distinct nonzero counts, translation or
planar rotation, and outward or inward winding. The tuple order is
`edges_fixed`, `degenerate_facets`, `facets_removed`, `facets_reversed`,
`backwards_edges`.

| Saved input | OrcaSlicer output | Ares at `5f0cedc9e` |
| --- | --- | --- |
| Zero counts, affine placement | Archive with `(0, 0, 0, 0, 0)` | Archive with matching counts |
| `(2, 3, 5, 7, 11)`, translation, outward | Archive with all five supplied counts | Archive with all five counts reset to zero |
| Same counts, translation, inward | Archive with outward winding and all five supplied counts | Outward winding, but counts reset to zero |
| Same counts, affine placement, outward or inward | Archive with outward winding and all five supplied counts | Exit 243 (`saved model metadata or transforms`); no output archive |

## Ares destination and scope

- `project/load/volume_metadata.rs::select` already selects a part and
  defaults absent counts to zero. Attach the selected typed counters to the
  mesh constructed in `project/load/mesh_prepare.rs`. The narrow handoff
  is the existing prepare call in `project/load/assemble/volumes.rs`; it
  must supply the selected counters alongside the mesh source. This does
  not require changing the volume constructor or `project/domain.rs`.
- Own the live counters with the vertices and triangles in
  `project/domain/mesh.rs::MeshGeometry`. The existing winding check must
  leave the counts unchanged. `project/domain/shared_mesh.rs::materialize_meshes`
  already shares the owner geometry, so a reused volume inherits its mesh's
  counts by the same operation. Do not add a second cache or change mesh
  ownership, selection, winding, or transformations.
- For the full saved CLI archive only,
  `project/export/sliced.rs::CliSavedSlicedProject::export_inner` already
  hands the loaded project to `project/export/saved.rs::from_saved_project_with_extras`.
  Project the live mesh counters into the saved `Part` on that path;
  `project/export/xml.rs` already writes the attributes in the required
  order. Remove only the all-zero counter predicate from affine admission.
  Keep the rest of the [saved affine constraints](saved-model-metadata-transforms.md)
  and all existing plate, part, provenance, and output checks. Do not reread
  or copy the source XML stats in the saved writer.

The scope is the counters already accepted by the saved part parser. Changes
to XML parsing or integer validity, STL repair, other mesh metrics, and
object or part selectors are out of scope. Keep the independent scene export,
unsliced export, `--min-save`, and the processed-plate compatibility entry
point unchanged. Do not introduce a public API or change slicing.

## Verification

Exercise the same complete route with a synthetic saved prism: a zero-count
affine control, distinct nonzero counts on translation-only and affine
instances, and reversed input winding with nonzero counts. Compare the
archive members against the pinned Native reference, assert all five counts
and outward output winding, and verify the command writes only the requested
archive. Cover the mesh-reuse rule by checking that a later volume with
different input counters uses the shared owner's live counters.
