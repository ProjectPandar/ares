# Spec: Task 22O241 layer-height processor synchronization

## Observable contract

At each project layer change, the processor's tracked extrusion height is synchronized to that layer's nominal height before the first path. The explicit layer prologue remains the sole nominal `; LAYER_HEIGHT:` line; the first ordinary perimeter does not repeat it. Later non-nominal bridge transitions still emit height tags. For current geometry this produces 515 tags; the Orca fixture has one additional bridge-to-nominal transition pair, deferred with bridge collection geometry/order.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode.cpp:4629-4633` together with `_extrude` at `6799-6804`: layer setup assigns `m_last_height = height`, while path emission uses the source `EPSILON` threshold. Destination: `project_slice/gcode_emit.rs` and `gcode_emit/motion.rs`. This corrects task 22O239's over-broad persistence interpretation. Deferred: the remaining bridge transition count and geometry/order differences.
