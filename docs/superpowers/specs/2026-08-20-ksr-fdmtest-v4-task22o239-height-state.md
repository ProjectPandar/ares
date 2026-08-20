# Spec: Task 22O239 persistent extrusion-height processor state

## Observable contract

For the current generated geometry, this removes 438 duplicate tags: the KSR stream contains 520 height tags rather than 958. Variable-width internal-bridge geometry is a deferred next slice; it accounts for the remaining three tags above the 517-tag Orca reference.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode.cpp:6794-6804`: `m_last_height` is persistent G-code processor state and changes only when `_extrude` observes a different path height. Layer-change metadata remains owned by Ares's layer prologue. Deferred: seam placement, variable-width geometry values, and later G-code differences.
