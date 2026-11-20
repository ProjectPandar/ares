# Normal raft interface arcs

Each project contains a synthetic 16-sided prism, 10 mm across and 1.2 mm high,
with triangulated caps and sides. The X1C and MK4 projects use their respective
0.20 mm Standard process and generic PLA presets. Both have three raft layers,
normal (auto) support type, and no additional object support. Each enabled/disabled
pair differs only in the `enable_arc_fitting` project setting. The Native streams
must extrude the raft interface at Z0.5, using arcs only when fitting is enabled.
