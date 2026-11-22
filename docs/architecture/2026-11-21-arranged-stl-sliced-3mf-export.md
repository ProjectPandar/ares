# Arranged multi-STL sliced 3MF export

OrcaSlicer v2.5.0 arranges direct STL inputs in `OrcaSlicer.cpp:5011-5065,5619-6005`.
Its CLI builds the sliced plate list and passes the placed model, settings,
and plate records to `Slic3r::store_bbs_3mf(StoreParams&)`
(`OrcaSlicer.cpp:7340-7371,8081-8086,8377-8405`;
`libslic3r/Format/bbs_3mf.cpp:9723-9734`).
`PartPlateList::store_to_3mf_structure` (`slic3r/GUI/PartPlate.cpp:8422`)
materializes the per-plate metadata.

For `--arrange 1 --slice 0 --export-3mf` with machine, process, and filament
settings, the CLI forwards the placed model and every completed plate from
`project_slice/cli_stl.rs` to `project/export.rs::write_package`. The existing
single-STL and saved-project paths remain compatibility shells around the
same `store_bbs_3mf` boundary. Export neither re-arranges nor independently
slices its inputs, and publishes no partial archive on a later plate failure.

A two-STL CLI witness checks plate identity, sliced members and payloads
against Native, and asserts that Ares writes every plate checksum before any
G-code. For ZIP member ordering and the comparison contract, see
[Sliced 3MF parallel member order](ard-sliced-3mf-parallel-member-order.md).

Mixed saved-project and STL inputs, explicit plate selection combined with
arranged multi-STL export, additional transforms, modifiers, painted meshes,
non-model volumes, and custom archive members are outside this slice.
