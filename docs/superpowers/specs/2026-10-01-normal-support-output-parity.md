# Normal support complete-output parity

Owning upstream boundary: `PrintObject.cpp::generate_support_material` /
`_generate_support_material`, `Support/SupportMaterial.cpp::
PrintObjectSupportMaterial::generate`, `Support/SupportLayer.hpp`,
`Support/SupportCommon.cpp` and `GCode.cpp::collect_layers_to_print`.
Destination: `ares-core::project_slice/support_material/` and the existing
`gcode_emit/layers/` object/support scheduling and emission boundary.
Object-only and raft scaffolding is a temporary compatibility shell.
Do not design an independent Ares support pipeline.

Prerequisites include `Flow.cpp::support_material_flow`,
`support_material_1st_layer_flow`, `support_material_interface_flow`, and
`SlicingParameters::create_from_config`'s support-nozzle limits and gap
assignment. Reuse these native contracts in the existing raft caller;
public complete-output raft probes cover first-layer width fallback,
automatic widths and zero/synchronized gaps without private/source pinning.
Native errors and remaining byte differences are failures, not completion.

The zero-top-gap dependency also includes the native
`SupportParameters::contact_fill_pattern` selector and
`SupportCommon.cpp::generate_toolpaths`'s raft-contact filler selection.
Rewrite into `raft/fill_params.rs` and `raft/fills/patterns/` using native
`FillConcentric::_fill_surface_single` and `FillGrid::fill_surface`
contracts. Preserve scaled-integer spacing, float density division,
outside-in contour order and nearest-point splitting; do not reuse a
similar filler whose numeric domains differ. Existing raft contact plans
are only the compatibility caller of these upstream support contracts.

Preserve `FillBase.cpp:2464-2466` first-call connection cost thresholds in
private per-job `SlicerContext` state shared across actual callers and
objects. The reference contract is one fresh native process per job, not
native long-lived process-global state. Keep other connection quantities
per-call and prove complete-output task isolation with repeated inputs
around jobs using different support spacing.

Generate complete output matching native Orca for both enable_support
Boolean values on the existing Ender-3 overhang input. Enabled normal(auto)
grid support must produce actual support X/Y motion with positive relative-E
extrusion, not merely an option string, role comment or matching header.
Disabled support must not produce support roles. Persist both sides and
the exported 3MF before output assertions. Strict ordered-byte comparison
excludes only generator identity/time and bypasses no differing commands.

Include upstream normal-support contacts, projections, independent heights,
base/interface regions, generated paths and object/support Z merging needed
by this fixture. Defer adjacent tree/organic generators, enforcers/blockers,
multi-material support and remaining styles/domains to source-cited slices;
do not count these domains complete. Existing general support exception
classifications may report missing work but cannot satisfy this regression.

Require witnessed public RED/GREEN, separate test modules, sub400LOC Rust
files without include-based splitting, full Nextest workspace, fmt/Clippy,
four-platform evidence and independent six-axis approval before the normal
support completion commit and push. Completed dependencies have a separate
ARD0085 checkpoint with their own bounded gates, not a replacement for this
ordinary-support exit. Global all-option/default/printer parity remains
open until its full original acceptance criteria are proved.
