# Frozen input-only regional ZAA LockedZag direction fixture

Approved SPEC f743713e799699c14d3eeb765907e50ae313c4e3.
Source: the admitted FULL14 main c251aa616 input for
orca_parity_locked_zag_shifts_preserve_complete_output
(option_coverage/fill_grid.rs:30-39): wide slab height2.4mm, LockedZag40%,
relative E, infill angle45, shift0.731272, wall_loops0, one top/bottom shell.

Mesh/profile members are preserved. Only project settings' ZAA switches
vary; min_z0.05 and minimize_perimeter_height35 are shared existing values.
ZIP metadata is fixed at2026-10-15T00:00:00. No Native outputs are included.

| Input | ZAA | dont_alternate | SHA256 |
| --- | --- | --- | --- |
| flag-false.3mf |1|0|ff30c4562351cddc26f6bbb6c534866ffa21212dae3cf54808a3a8792c30ad43|
| flag-true.3mf |1|1|d5811d559eecf41d27a099a8b1909352b76c1e9cc4b33a7e90873733ba791610|
| disabled-zaa-flag-true.3mf |0|1|31b548284c20706fd37a2c2e1aa80d80dc442d8e11fcdfa06ae192c49784b474|

Fill.cpp::Layer::make_fills:1235-1242 sets the regional runtime flag.
FillRectilinear.cpp:3907/3927 uses that same filler for LockedZag skeleton
and skin. FillBase.cpp::_infill_direction:311-313 conditionally alternates.
The public witness is Sparse infill on physical layer2 (object layer1).
Assert its exact start/end XY and relative E before unchanged strict whole
output assertions. Both controls and the active case run through API+CLI.

Before RED, fresh official+seeded Native repeats accepted all cases and
changed enabled-flag0/1 Sparse geometry. Ares flag1 retains Native flag0
geometry. First motion delta is the entry travel to layer2 Sparse infill;
its previous TYPE is Skirt until the new Sparse TYPE, not a skirt delta.
An earlier M73 placement delta is retained by the strict comparator:
GCodeProcessor.cpp:1232-1257 computes progress from elapsed/total time.
