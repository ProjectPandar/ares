# Type1 wipe tower arc-fitting mode

Upstream: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, `GCode/WipeTower.cpp`. `WipeTower` enables arc fitting only when `enable_arc_fitting == ArcFittingMode::Enabled` (:2017). The resolved flag is passed to every `WipeTowerWriter` (:2324, 2485, 2948, 3535, 3659, 3817, 4010, 4126, 5251).

The writer handles two inputs:

- `polygon()` (:1122–1130) closes the contour and simplifies it at 0.001 mm before applying the selected mode.
- `generate_path(Polylines&)` (:1382–1388) applies the selected mode directly to each path.

In both cases enabled mode uses arc fitting followed by simplification. Other modes use linear Douglas–Peucker simplification and replace the fitting records with one linear range per path (`Polyline.cpp:155–165`). Both branches use 0.1 mm. The first-layer rib origin is derived from the mutated paths after simplification (:5382–5385), not the raw rib polygon or the brim box.

The Rust boundary is `gcode_emit/wipe_tower/type1/{generator.rs,writer.rs,writer/path.rs}`. Reuse the writer factory, the existing linear and arc simplifiers and segment materialization. Replace the unconditional fit with the upstream selection in both consumers, without changing their entry-point selection, ordering, width or offset arithmetic. Planner, raw footprint/fillet/gap geometry, skirt and travel routing, general motion, and Type2 remain outside this port.

Public regression cases slice runtime-derived Type1 projects: the rib wall on the committed SEMM R1 geometry and two differently assigned cubes using stock Qidi Q2 0.4 presets. A mode-enabled control must preserve Native agreement. Compare ordered Native, API and CLI output with only generator identity normalized; if a separately identified upstream parity gap blocks whole-file equality, compare the affected tower span without normalizing motion and record the whole-file first difference. Generated inputs and Native outputs are not versioned.
