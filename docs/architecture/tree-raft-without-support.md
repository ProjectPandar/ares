# Organic tree raft without object supports

## OrcaSlicer boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `PrintObject.cpp:998–1002` calls `_generate_support_material` for a raft on a nonempty object even without ordinary support. `PrintObject.cpp:5088–5093` dispatches tree support by type.
- `Support/SupportParameters.hpp:215–221` resolves the default tree style to Organic. `Support/TreeSupport.cpp:1901–1908` dispatches Organic to `generate_tree_support_3D` before the non-Organic tree processing and toolpaths at `:1910–1951`.
- `Support/TreeSupport3D.cpp:3662–3673` creates the raft contact and invokes `generate_raft_base` for the raft-only case. `:1064–1082` places the optionally expanded first object slice directly as a roof; it does not rasterize that contact onto the normal support grid.
- `Support/SupportCommon.cpp:447–513` consumes that contact to construct a multi-layer raft footprint. For one raft layer, the contact remains the raft surface; `:539–571` handles first-layer expansion for support columns, not the Organic roof. `TreeSupport3D.cpp:3701–3703` uses the shared toolpath generator.
- By contrast, the normal route (`Support/SupportMaterial.cpp:530,602`) reaches the shared raft and fill functions with normal-grid contact geometry. The non-Organic `TreeSupport.cpp:1557–1604` raft path is different again and is not exercised by default Organic tree support.

## Rust destination

`project_slice/support_material/project.rs::generate` currently returns no support layers when object support and enforced layers are disabled. The G-code scheduler then uses the raft-only adapter in `project_slice/raft/bridge/project.rs`, whose `raft/polygons.rs::raft_polygons` samples the contact with `SupportGridPattern`. The resulting raft polygon is already different before the extrusion paths are ordered.

For an Organic tree object with a raft, admit the object to the existing Organic generator even when object support and enforced layers are disabled. `support_material/tree/organic/generation.rs` already has a raft-contact-only branch and suppresses overhang detection when object support is disabled. Installed support layers remain the scheduler's input; the separate raft adapter remains a temporary compatibility path for the unchanged routes.

Keep normal and non-Organic tree dispatch unchanged. The change does not port interface arc fitting, change the first-layer fill algorithm, or alter support extrusion ordering. Native X1 Carbon and MK4 slices of a synthetic cube exercise one contact-only raft layer and three raft layers including a base; without a raft or object support the same presets must emit neither support role. Compare complete public API and CLI output to Native.
