# Plan: Task 22O.126 layer interval precision

1. Add a focused `planned_layers` assertion proving interval subtraction remains in `f64`; run it red against the current pre-subtraction `f32` conversion.
2. Port OrcaSlicer `PrintObjectSlice.cpp::new_layers` by assigning `height = pair.hi - pair.lo`, and update obsolete expectations that pinned the narrower intermediate representation.
3. Run the focused layer test, regenerate the KSR project, verify the regular inner-wall volumetric feedrate, then run rustfmt and strict `ares-core` Clippy; commit and push this isolated source-cited slice.
