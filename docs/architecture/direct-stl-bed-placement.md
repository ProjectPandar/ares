# Direct STL import drops the instance to the bed

## Native boundary

In OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, `CLI::run` reads ordinary input files with `AddDefaultInstances` (`OrcaSlicer.cpp:1723–1758`). It preserves imported placement for the recognized Bambu/Orca project branch (`:1759–1768`); the other-file branch calls `ModelObject::ensure_on_bed()` after loading and adding any clones (`:1924–1979`). This includes direct STL; it is not conditional on `--load-settings` or on the later `--arrange` value. A foreign 3MF not recognized as a project can enter that same Native import branch, so the distinction is not simply the filename extension.

`ModelObject::ensure_on_bed` does nothing with `auto_drop=false` (`Model.cpp:1896–1920`). A fresh STL object has `auto_drop=true` (`Model.hpp:405`). Project 3MF can carry an explicit off value (`Format/bbs_3mf.cpp:2263–2288`). The default argument is `allow_negative_z=false` (`Model.hpp:519`), so a fresh STL is translated by `-min_z` along Z, for either sign. `Model.cpp:1632–1672` computes the minimum from model-part mesh vertices transformed by instance zero and the volume transform; it is not mesh-local Z or the center of the volume. The translation moves the instance, not the mesh vertices or XY.

`--arrange 0` disables arrangement after import, not the import drop (`OrcaSlicer.cpp:5051–5068`). The arrangement pass does not call `ensure_on_bed` itself. A separate pass after transforms and arrangement is conditional on `ensure_on_bed` (`:6273–6280`), a CLI option disabled by default (`PrintConfig.cpp:12946–12949`). Omitting/disabling that option leaves the early import drop intact. Its post-transform policy, including for project files, is outside this change.

## Rust boundary

In `crates/ares-core/src/project/domain/profile_input/stl.rs`, `Project::from_stl_with_profiles` imports a single raw STL volume and creates a default instance at identity. After constructing this project, reuse `drop_instances_to_ground` on that instance before profile placement, edits, orientation and slicing. This matches both the above-bed and below-bed paths without recentering XY or modifying the volume. `Placement::None` means no additional placement transform, not bypassing import. Do not change project 3MF loading, prepared plater models, multipart import, common STL decoding, or the later center/arrange behavior.

## Public parity witness

Translate the vertices of the existing 10 mm cube so the minimum Z is `+1 mm` or `−1 mm`, leaving XY and the selected Flashforge Adventurer A5 profiles unchanged. Use the same STL and flattened settings on both public CLIs; Native runs with `--arrange 0` and its default post-transform drop option. Before the change the above-bed Ares case rejects with an empty first layer and the below-bed case slices only 45 instead of 50 layers. Both inputs must match fresh Native complete ordered G-code (generator identity only) after import, and Native must retain the grounded 50-layer result. Run both official and seeded references with fresh evidence and compare the adjacent profile/placement inventory before and after.
