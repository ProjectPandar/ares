# TiQ2 outer-wall speed: source ownership conclusion

## Decision

Do not add another speed fix for the historical line1265 F2700/F2160
symptom. On main5128ead05 it is already resolved by3d65ec24e's upstream
small-perimeter caller boundary port. Full TiQ2 parity is still not achieved.

## Owning upstream boundary

`libslic3r/GCode.cpp:5809-5813::extrude_loop` applies small-perimeter speed
only to a complete loop. `:6038::extrude_multi_path` does not infer small-loop
speed from an individual path's length. `_extrude` chooses the role speed
when its caller passes-1, then `:6657-6718` uses
ExtrusionQualityEstimator::estimate_extrusion_quality for variable overhang
speeds. `:7112-7209` emits those processed-point feed rates.

The target is an ExtrusionMultiPath external perimeter, not a tower path.
The existing Rust destination is gcode_emit/motion/{features,loop_paths,
materialized,path,overhang}. The deleted scaffold behavior classified each
short path as a small loop. At outer_wall_speed120 and small_perimeter_speed
30%, that incorrectly capped it to36mm/s (F2160). No additional Type2,
ToolOrdering, cooling multiplier or fixture condition belongs here.

## Native confirmation

Gdb on the existing exact seeded Native binary confirms the target path
starting at(4874803,4816747) enters `_extrude` from extrude_multi_path with
caller speed-1. The quality estimator receives reference120 and original120;
GCodeWriter then receives F2700 and F4560 from `_extrude`. Thus these values
exist before CoolingBuffer; F2700 is not a cooling reduction of F2160.

The complete output of the debugger run equals the seeded oracle after only
banner normalization:816581 normalized bytes, SHA256
8e417e46deaef9e48f9390cf9ae0412b7977722b7722fdfcf56c2eb7cd467001.
No OrcaSlicer rebuild or Native artifact commit is required.

## Current complete-output boundary

The original OrcaRunner whole-file comparison rerun on5128ead05 agrees
through1822 lines, including the historical line1265 feed rates, segmentation
and E values. The first difference is now line1823, inside tool-change output:
Native `M106 S204`, Ares `M106 S130`, immediately before T1. Expected/actual
normalized lengths are816585/816551. The complete comparator remains strict;
no residual output is filtered or declared green.

## Proposed next rewrite slice (approval required)

Keep speed selection unchanged. First identify the owning GCode tool-change
fan/CoolingBuffer state transition that supplies the M106 value, using the
same input and existing Native debugger. Only after confirming that boundary,
define the exact Rust fan-state destination and write a runtime-OrcaRunner
RED before any production change. Include that confirmed transition only;
defer independent skirt hull discrepancies and other adjacent behavior.
Native-generated streams and debugger maps remain runtime artifacts, not
repository fixtures. The existing3mf input is sufficient.
