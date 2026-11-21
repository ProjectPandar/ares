# Arachne loops with disconnected sampled paths

OrcaSlicer 2.5.0, `src/libslic3r/PerimeterGenerator.cpp:572–579`, notes that sampled paths in a closed Arachne extrusion can be disconnected and trigger its debug assertions. The release build orients and appends the sampled loop without repairing it.

In `project_slice/perimeters/arachne/traverse.rs`, use that same behavior in every profile: omit the two debug checks specific to the closed-loop branch. Keep the existing orientation, loop classification, and handling of open extrusions. Do not join or otherwise modify sampled paths.

A synthetic tower with asymmetric overhangs on stock Prusa MK4 presets must match the complete Native G-code through the public slicing API and CLI in the default debug test profile. Compare separately against official and seeded Native runs, normalizing only generator metadata.
