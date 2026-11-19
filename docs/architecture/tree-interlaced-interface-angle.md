# Interlaced interfaces for non-Organic tree support

## Native boundary

OrcaSlicer 2.5.0 `Support/TreeSupport.cpp::generate_toolpaths` resets the fill state for each area group and selects the interface angle from that group's ID (`:1697–1702`). The first roof layer uses the same selector (`:1728–1731`). Floor and roof use fixed-angle rectilinear fill for `rectilinear_interlaced` (`:1749–1758`, `:1765–1774`).

`Support/SupportParameters.hpp:121–122,308–327` defines the interlaced angle as `interface_angle − π/4` for even IDs and `interface_angle + π/4` for odd IDs, where `interface_angle` is the configured support angle plus 90°. `TreeSupport.cpp:2614–2645` counts roof (including the first roof layer) and floor independently. The filler does not alternate the selected angle again (`Fill/FillBase.cpp:449–457`).

## Ares boundary

In `project_slice/support_material/tree/non_organic/toolpaths/interfaces.rs`, replace the roof/floor interlaced-only `base_angle + (interface_id & 1) × π/2` with the existing `toolpath_parameters.rs::ToolpathParameters::support_interface_angle`, matching the first roof layer. Keep the current per-group ID assignment, fixed-angle rectilinear fill and path ordering. This change does not address other interface patterns, Organic or normal support, raft interfaces, tree geometry, or progress emission.

## Verification

Start from `8b2829635` plus the extra tree walls by area overlap change at `39cce5086`; the latter is necessary for strict parity on the ordinary Slim overhang control independently of interface angle.

Compare full ordered G-code from Native and the public slice API with only the generator line normalized, and require raw API/CLI equality. Use a half-scale overhang and a ledge for the initial roof layer and the following roof, a printed shelf for the first floor interface, and both Native modes. Run the same support cases before and after the angle change to detect regressions.
