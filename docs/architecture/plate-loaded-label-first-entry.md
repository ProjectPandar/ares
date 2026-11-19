# First loaded instance label per object on a 3MF plate

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) declares `PlateData::obj_inst_map` as `std::map<int, std::pair<int, int>>` in `libslic3r/Format/bbs_3mf.hpp:100`. `_BBS_3MF_Importer::_handle_end_config_plater_instance` (`bbs_3mf.cpp:4986–5004`) uses `emplace(object_id, (instance_id, identify_id))`: the first XML entry for an object on each plate wins, even when its ID is zero. The importer subsequently assigns `loaded_id` only to the instance named by that entry (`:2466–2499`). A missing instance takes no assignment and does not promote a later entry. Loaded-label mode is enabled by the CLI (`OrcaSlicer.cpp:6575–6585`); `ModelObject::get_instance_label_id` follows the constructor identity path when no nonzero loaded ID is assigned (`libslic3r/Model.hpp:1437–1443`).

## Rust boundary

In `project/load/metadata.rs`, select the first entry per object per plate for the loaded-label index used by `project/load/assemble.rs`. Preserve every parsed plate instance and its metadata for plate routing and public project inspection. The build item loop and constructor identity path stay unchanged: non-selected instances still slice with their original transforms. The tuple lookup is an assembly adapter for the selected entries, not the upstream uniqueness rule.

This port does not change plate membership, geometry, firmware label formatting, or which identity source the saved-project skip option uses. In particular, an `identify_id` on a later entry for the same object is not a loaded ID and must not match loaded-ID selection.

## Observable behavior

Slice synthetic 3MF projects with two build instances of one box using the pinned Native producer and Ares, comparing ordered G-code with only the generator line normalized. With X1C object exclusion enabled, the first metadata entry supplies the saved label; the other instance has its constructor label. The result must hold for identical poses and for a rotated and scaled second instance. Reversing the two XML entries without changing the build items changes which instance receives the saved label. A zero ID in the first entry must not allow the second entry to supply a saved label. Use K1 as a control for normal object exclusion and verify through the public project import API that neither instance nor its plate membership disappears.
