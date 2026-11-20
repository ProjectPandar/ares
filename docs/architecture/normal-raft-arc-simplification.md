# Normal raft arc simplification

Source: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

- `Support/SupportMaterial.cpp:527–530,570–575` constructs normal raft and installs its support layers. `Support/SupportCommon.cpp:1554–1657,1759–1761,1876–1891` generates raft base, interface and contact toolpaths as support extrusions.
- `PrintObject.cpp:1093–1104` simplifies each installed support layer. `Layer.hpp:322` and `Layer.cpp:361–425` apply the support rule recursively to paths, multipaths and loops: with arc fitting Enabled and no spiral mode, use `SCALED_SUPPORT_RESOLUTION` (`libslic3r.h:91–92`, 0.0375 mm); otherwise apply plain simplification at the configured resolution.
- `ArcFitter.cpp:103–152` uses the same tolerance for fitting and for the subsequent Douglas–Peucker simplification of each fitted or linear span. `GCode.cpp:10058,10123–10173` consumes the fitted result rather than trying to refit the support path.

The raft-only schedule in `project_slice/gcode_emit/layers/raft_schedule.rs` temporarily materializes raft support as fill entities. Those entities reach the existing simplifier but incorrectly pass the print resolution even in arc mode. Use the same tolerance selection as `project_slice/path_simplification.rs` uses for installed support, only for normal raft in this adapter. Keep object paths, ordinary installed support, periodic spiral handling, and the arc writer unchanged. Tree raft contact generation and the tree adapter are outside this change.

On a normal X1C raft with three layers, reducing the Native support tolerance alone from 0.0375 mm to 0.012 mm replaces the first support-interface arc at Z0.5 with the two linear moves emitted by the raft adapter. Neither the path generator nor the writer changes. Exercise the rule through whole-project slicing with arc fitting both enabled and disabled, on X1C and MK4, using the unchanged whole-stream comparator.
