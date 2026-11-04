# SPEC: Repetier temporary jerk M207 ostream precision

## Upstream rewrite boundary

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:262,272-285::set_jerk_xy` uses plain
ostringstream and streams Repetier `M207 X` with double jerk_xy after clamps,
therefore defaultfloat six significant digits. Rust destination only
`gcode_emit/motion/jerk.rs` Repetier arm33-35, currently axis(x). Reuse existing
format::ostream, not new formatter/helper/configpolicy. This writer shell follows
upstream, not an independent Ares quantization rule.
Included: ONLY Repetier M207 field axis→ostream. Deferred: cache/clamp/tolerance/
flavorgate, comments, other flavors, formatter implementation, acceleration,
processor/templates/all other owners. Klipper separate set_jerk_xy precision is
a separate follow-up, expressly NOT included here.

## Reachable evidence

StockEnder3/toweroffRepetier, sole real default_jerk option varies7/7.12345/7.125.
GUI Tab2566/PrintConfig3289-3296 coFloatmin0; valid saved floats, no assertion
about displayed decimals or fine physical tuning. Other profile/geometry/templates
unchanged. Repetier machine-limit gate false caches 0,
so no clamp/roundingconfound. Target NativeM207X7.12345 vsAres7.123, firstdiff4911;
controlsfullPASS,6freshNative slicesstable/allCLIexit0.

SHA:
- integer7:dd456257226ef4563c4742764fee803e47fc459568f31da39f59413441e6d626
- target7.12345:1029964bc3ef6b4457d215249a85db6c75692f7dcdc22cb77ff307338f420076
- milli7.125:ea68cf109f4abd9ff14966be8eeb9627fc179d258cb11ab4991b00cd1b274201

## RED-first failures / exits

Freeze3inputs, direct existing native_stream_case calls, no helpers. FreshNative
replay then callback reads defaultjerkfrominput and observes actualNativeM207
word retaining that value, no hardcodedNativecoordinate/count/valuegolden.
Then complete unmasked API/CLI comparison with existing generator-only seam,
all timing/config/footer included; no Nativeoutputscommitted.
Failureclasses:fixed3vs6significant,wrongcommand/flavor,cache/admission/clamp
change,globalformatterchange or accidentalKlipperfix. Commit REDbeforeproduction,
onlytargetFAIL/twocontrolsPASS bothmodes/Native replay+API==CLI qualify.
GREEN one existingformattercall,tests/inputsfrozen. Official then
seeded, fmt/workspace all-target clippy/3 directed both PASS.
No lint suppressions; group registration without squeeze,
all touched Rust ≤400 LOC. Preserve all previous slices unchanged.
