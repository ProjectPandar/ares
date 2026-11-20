# Direct CLI: checked G-code path conflicts

Upstream: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
`libslic3r/Print.cpp:3946–3964` runs `ConflictChecker::find_inter_of_lines_in_diff_objs`
when checking is enabled. It passes the object paths even without a wipe tower;
the tower is an optional input. Reuse the Rust path collector and intersection
checker in `project_slice/gcode_emit/conflict.rs`. The direct CLI enables the
object/support-only case as well as the existing tower path. Keep the scope of
checks outside the direct CLI unchanged.

`OrcaSlicer.cpp:7134–7138` consumes a detected conflict before processing any
slicing warning or exporting G-code. The direct CLI writes the resolved plate
settings but no sliced-plate entry or slicing warnings, logs the conflict at
error verbosity, and exits with `CLI_GCODE_PATH_CONFLICTS` (`Utils.hpp:93`,
report `-101`, POSIX exit `155`). The diagnostic is not stderr. When
`--no-check` precedes `--slice`, skip conflict checking entirely; a later flag
does not change the result. This does not turn off validation or export checks.

The shared direct slicing-warning consumer follows `OrcaSlicer.cpp:7141–7173`:
log each delivered critical slicing warning in order at warning verbosity,
including the action-local `no_check` value. Only `EmptyGcodeLayers` and
`GcodeOverlap` reject the checked slice. Do not add an overlap warning producer:
upstream declares the type but does not produce it at this pin. Default status
notifications do not enter this consumer. Leave strict handling of noncritical
warnings unchanged.

The synthetic crossing-gap project duplicates the existing `odd-rotated.3mf`
object as `other-cube.stl` at a 3 mm offset along both XY axes, on the same
plate. With checking, crossing paths take precedence over the empty-layer
warnings. With checking skipped, both critical records appear in the log and
the last warning is retained on the exported plate. Run both binaries with
identical argv and compare exit, stdout/stderr, result JSON (elapsed times
only normalized), G-code presence, and the relevant diagnostic records.
