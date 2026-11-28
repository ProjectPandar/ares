# Saved previews and auxiliary pictures in sliced 3MF export

## Native seam

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) loads the
first positional 3MF with `LoadModel | LoadConfig | AddDefaultInstances |
LoadAuxiliary` (`src/OrcaSlicer.cpp:1723–1758`). After slicing,
`CLI::export_project` passes the selected zero-based plate, thumbnail data,
and the loaded model to `store_bbs_3mf` (`:8081–8086,8377–8405`). This slice
ports the saved preview and auxiliary handling of that `libslic3r` reader and
writer, without adding a renderer or a second archive path.

In `src/libslic3r/Format/bbs_3mf.cpp`, the importer extracts `Metadata/*.png`
when loading configuration (`:2124–2126,2966–2992`). The `thumbnail_file`,
`thumbnail_no_light_file`, `top_file`, and `pick_file` plate fields select
which files are used (`:2452–2456,4714–4728`); image names alone do not assign
a preview to a plate. In the unchanged-placement route with the complete
saved preview set, `CLI::run` uses these files rather than rendering
(`OrcaSlicer.cpp:7431–7504,7880–7928`).

All ranges below refer to `bbs_3mf.cpp` unless otherwise specified.

| Supported input | Output and source |
| --- | --- |
| Selected plate's referenced `Metadata/plate_<n>.png` (8-bit RGBA) | Decode to bottom-up RGBA (`OrcaSlicer.cpp:588–619,7893–7915`; `src/libslic3r/PNGReadWrite.cpp:137–207`). Write a new `plate_<n>.png` and derive `plate_<n>_small.png`, both stored without ZIP compression (`:6530–6542,6959–7024`). Do not write the original PNG bytes in place of the decoded/re-encoded primary preview. |
| Selected plate's referenced `Metadata/plate_no_light_<n>.png`, `Metadata/top_<n>.png`, and `Metadata/pick_<n>.png` | Keep the three PNG payloads unchanged, written with the selected plate's output names and ZIP Store (`:6595–6627,6912–6919`). The primary and small preview precede those three; the preview block precedes the first-layer plate JSON (`:6485–6667`). |
| `Auxiliaries/.thumbnails/thumbnail_3mf.png`, `thumbnail_middle.png`, and `thumbnail_small.png`; `Auxiliaries/Model Pictures/<file>.webp` and `Auxiliaries/Profile Pictures/<file>.webp` | Keep nonempty files by validated relative name and payload. `_extract_auxiliary_file_from_archive` checks the destination stays within the auxiliary root (`:2115–2119,2921–2964`). `_add_auxiliary_dir_to_archive` walks regular files, writing their relative paths before root relationships (`:6864–6893,9136–9189`). PNG uses ZIP Store and WebP uses Deflate (`:6912–6919`). Preview folder names are defined by `src/slic3r/GUI/Project.cpp:448–453`, and the cover names by `bbs_3mf.cpp:224–226`. |
| `Metadata/cut_information.xml` with only nil cut identities, zero connector counts and no connector records | Admit a well-formed inert document but omit the member from output. Load parses records indexed by the one-based object ordinal and applies their cut IDs and connectors (`:2062–2065,2321–2335,2718–2775`). An ID of zero does not make an object cut (`src/libslic3r/Model.hpp:577`, `ObjectID.hpp:36–37`). `_add_cut_information_file_to_archive` only writes a member when a cut object contributed to the tree (`:7883–7943`). Do not apply this omission to active or unrecognized cut data. |

The small preview is `128 × 128` (`bbs_3mf.hpp:33–34`). The native loop averages
each channel independently over integer-size blocks, truncating incomplete
right/bottom edges (`bbs_3mf.cpp:6976–7016`). This slice covers positive equal
horizontal and vertical step factors; inputs outside that shape do not justify
porting out-of-range indexing. The fixture must make both axis order and integer
averaging observable.

The writer regenerates the four `model_settings.config` preview references in
their existing field order after `gcode_file` (`:8537–8563`). Selected-plate
`_rels/.rels` retains the root model relation and points its thumbnail/middle
relationships to the emitted primary preview, and its small relationship to
the emitted small companion (`:7062–7115`). Do not reuse the input root
relationships. Keeping auxiliary cover PNGs does not change the selected
plate's targets. The existing plate JSON, G-code, checksum, G-code relationship,
slice info and filament sequence continue to be produced by the sliced writer.

## Ares destination and scope

The current catch-all in `crates/ares-core/src/project/export/saved.rs::from_saved_project`
rejects every member outside the model paths and ordinary saved configuration.
Replace that catch-all only for the recognized kinds above in the saved
single-plate CLI `--slice 1 --export-3mf` route. Interpret preview references
from the already loaded `project/model_settings.rs::PlateSettings`, reuse
`ProjectArchive` and `PackagePath` in `project/export/sliced.rs`, and carry the
resolved assets into the existing `project/export.rs::write_package` and
`project/export/xml.rs` writer. Keep ZIP entry and decoded image limits and
validate XML and package paths; do not extract to a host path or add another
archive reader. Reuse the existing miniz-style RGBA encoder in
`gcode_thumbnails/png.rs` rather than introduce a second PNG encoder. Use
file-backed-member ZIP fields for kept auxiliary and secondary PNG assets,
and memory-member fields for generated images, following the writers above.

Native walks auxiliary directories breadth first without sorting siblings.
Keep the auxiliary block in the same position and use source archive order
as a stable sibling tie-break in the in-memory breadth-first traversal. Do
not infer a cross-filesystem sibling ordering promise from the Native walker.

This does not relax existing unknown-member, version, mesh, transform, or
preset admission. No edits to plain slicing diagnostics, unsliced export,
STL, browser/WASM, or packages without these extras. Deferred: partial or
stale preview sets requiring rendering, other primary PNG layouts, `--min-save`
with these extras, other plate selections, other auxiliary path or payload
kinds, embedded previous G-code, and active cut-geometry or connector
serialization. Keep the prior unsupported-input behavior outside the stated
shape rather than silently dropping data.

## Public CLI verification

Extend the synthetic saved input used by `--slice 1 --export-3mf` tests. Add
hand-authored preview metadata and independently generated PNGs; use a
patterned RGBA image with positive equal downsample factors and non-native
encoding, so a raw copy fails. Generate separate secondary PNGs and small
WebP pictures with invented names (including a UTF-8 name),
and an inert cut document with a nil identity and no connectors. Do not use
external project data or names.

Exercise the complete set of four plate previews together, with the inert
cut document and one auxiliary file per case; parameterize the auxiliary
case over the cover and picture kinds. A singleton auxiliary block compares
exact order without depending on the enumeration order of Native's filesystem.
Require successful exits, both archives and no standalone G-code. Compare
member names, multiplicity, placement, stable ZIP metadata, and uncompressed
payloads against live Native, including all links, the regenerated small PNG,
and absence of `cut_information.xml`. Keep the existing sliced archive
comparator unchanged, including its banner/checksum handling and existing
parallel spans. Do not add payload masks. Retain the no-asset positive and
unsupported-shape controls in the paired export regression cohort.

Run committed test-only and final focused cases in the Official and Seeded
v2.5.0 lanes; pair the adjacent saved/sliced export cohort against the parent
in both lanes, and run the native and WASM static checks on the exact tip.
