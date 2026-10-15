# Loaded filament flow admission fixtures

Input-only 3MFs; no Native output is committed. Source boundary:
`OrcaSlicer/src/libslic3r/PrintConfig.cpp:2315-2325,10449-10454`.
Derived from clearance-admission/control.3mf; only project_settings.config's
filament_flow_ratio differs. Manifest records exact input SHA256.

Singleton filament family: control [1], zero [0], upper-control [2].
Multi-value/cardinality expansion is deferred, not claimed as verified.
Runtime Native runs first. Zero must produce exact configuration rejection:
`filament_flow_ratio: invalid value 0`, CLI -18/POSIX238, no G-code;
the validated API must return matching InvalidValuesInProject.
The public RED fails on current API acceptance until a separately approved GREEN.
Both controls compare complete ordered Native/API/CLI bytes using the existing
generator-only comparator, and require raw API==CLI.
Runtime artifacts include Native invocation, diagnostics, outputs and comparison.
