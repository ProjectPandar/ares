# AutoBrim fidelity notes

The AutoBrim implementation follows the upstream grouping and width boundaries. Volume grouping uses `PrintObjectSlice.cpp:517-660` and `:798-807`: only `model_part` volumes participate in positive groups, negative volumes are subtracted, and first-layer `lslices` are assigned to groups after simplified comparison while the original polygons are retained. The three safety offsets use Clipper's default miter limit 3 (`ClipperUtils.hpp:19-27`).

Width calculation follows `Brim.cpp:260-305` and `:497-527`: group height comes from the transformed model-volume mesh bounding box, the 2-D first-layer polygon box feeds the height-to-area term, group width is quantized before the `<5`/`1.5*thermalLength` decision, and the additional-width and bbox clamp are evaluated per polygon. Grouping resolution is the scaled print resolution used by `:798-803` without an extra clamp.

`Model::findMaxSpeed` (`Model.cpp:3222-3255`) is an object-config override over the global `printSpeedMap`; resolved region speeds are not assumed equivalent. The current path matches the single-filament resolved configuration used by the parity fixtures. Multi-filament volume-specific adhesion and thermal-length selection (`Brim.cpp:120-148`, `Model.cpp:3258-3280`) remains an explicit follow-up: a mixed-filament group must be fail-closed or port the per-volume tables before it is admitted.

The modifier-volume witness was removed after Native-first probing: legal modifier inputs produced no `;TYPE:Brim`, so no active Native observable could distinguish grouping or adhesion. The retained multi-volume witness starts from the Native-active bump base and records a changed Brim block after adding a second volume.

The former PLA/PETG material-table whole-output case is excluded from this
slice's target set because its first difference is the pre-existing
GCodeProcessor M73 placement/time-estimator gap (Native `M73` precedes the
first Ares move; the same offset occurs on the pre-AutoBrim parent). The
AutoBrim grouping witness remains the multi-volume bump-base comparison.
