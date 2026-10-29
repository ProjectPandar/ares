# ByObject transition comment inputs

Input-only 3MFs; no Native outputs. Source:
GCode.cpp:3321-3322, GCodeWriter.hpp:266-267, GCodeWriter.cpp:678.

comments-on is the unchanged discovery combined-control input; comments-off
changes only gcode_comments in project_settings.config. Two separate 4mm-high
objects, ByObject, combined skirt, one filament, no brim/support/raft/tower.
combine_brims=0 is registered in current pinned PrintConfig.cpp:1744 and was
verified in both Native effective footers, not assumed from a generic release.
Manifest freezes SHA256. Runtime Native runs FIRST. Both cases assert whole
ordered Native/API/CLI equality and raw API==CLI; comments-on is the intended
RED (actual empty/missing transition comments), comments-off is a strict control.
No skirt guard change, comment stripping or inverted divergence assertion.
