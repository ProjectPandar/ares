# Spec: Air-filtration option activation

Follow ARD-0048's `PrintConfig`, `GCode::_do_export` and
`GCodeWriter::set_exhaust_fan` boundaries and Rust destinations.

The existing sweep leaves filament exhaust options inert and routes the
printer capability through support-material activation. Correct the six
selected domains without changing unrelated support classification.

Acceptance: all four Boolean domains exhaust 0/1, both integer speeds
exercise 0/100/seeded interior, exported settings preserve the tested
value and dependencies, and original output contains exactly the expected
ordered `M106 P3` commands. All 14 complete outputs must match Ares.
Disabled gates suppress only the applicable commands, zero speed emits
an explicit stop, and the two phases remain distinguishable.

Retain strict byte comparison and full artifacts. Single-filament cube
coverage does not certify multi-tool maximum selection, custom templates
or full-comment mode. Existing compatibility scaffolding only validates
the named upstream rewrite; it is not a new slicing design.
