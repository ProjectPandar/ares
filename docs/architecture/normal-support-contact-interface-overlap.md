# Normal support bottom-contact overlap with a top interface

The pinned OrcaSlicer v2.5.0 `Support/SupportCommon.cpp:1805–1812`, in `generate_support_toolpaths`, distinguishes two same-height, compatible cases. With bottom interfaces enabled and a `BottomInterface` layer, it merges that layer into the bottom contact. Otherwise it subtracts the interface's `polygons_to_extrude()` from the bottom contact's `polygons_to_extrude()` and stores the result only as the remaining extrusion domain. In the subtraction case neither source polygon list changes. The interface still owns its extrusion area.

`SupportGeneratorLayerExtruded::{polygons_to_extrude,set_polygons_to_extrude,polygons_append}` (`SupportCommon.cpp:909–916,953–955`) separates the remaining extrusion domain from the source polygons. Interface fill consumes the former (`:1852–1897`); support-island collection consumes the latter (`:1978–2017`). Unconditionally merging compatible layers instead applies a safety offset to the source union (`:927–950`), can remove a small boundary lobe before clipping Grid lines, and can change whether a travel is contained in a single support island.

## Rust boundary

In `project_slice/support_material/toolpath_cache/contacts.rs::prepare_grid_contacts`, replace the unconditional bottom-contact/interface merge in the compatible non-organic branch with the source distinction above. Use the resolved bottom-interface count and the selected interface layer's `SupportLayerType`, not a fill pattern or printer check. Use the existing `ExtrudedLayer::{polygons_to_extrude,set_polygons_to_extrude}` for the subtraction, without a safety offset or changes to `SourceLayer` polygons. Reuse the existing merge for the compatible bottom-interface case. Leave top-contact arbitration, support generation, clipping kernels, connector construction, island assembly, travel predicates, and Organic/tree behavior outside the slice.

## Verification

Compare the complete public slicing output with the pinned Native executable, using the existing ordered-byte comparator, for three single-object normal-support projects: X1 Carbon with Grid interfaces, X1 Carbon with two raft layers, and K1 with a rectilinear-grid base. The first exercises a Grid connector over a retained contact lobe; the latter two exercise retraction over support-island holes. Each runs with both Native launch modes. Also compare the affected support controls and option matrix before and after, with no passing case regressing.
