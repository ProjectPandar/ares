# Cooling modal feedrate after pressure equalization

## Upstream boundary

Pin OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
`src/libslic3r/GCode.cpp:5241–5248` schedules pressure equalization before
cooling. `GCode/PressureEqualizer.cpp:483–520,930–955` passes through unchanged
input and emits tagged speed lines for adjusted extrusions, quantized to
1 mm/s. Cooling must accept both the quantized values and the fractional
values that survive pressure equalization.

`GCode/CoolingBuffer.cpp:894,983–1019` starts a fresh integer modal feedrate
for each buffered layer. It obtains the comparison value with `atoi` from
an unchanged F word, or rounds the computed feedrate for a slowdown. On an
equal value it skips zero-length or adjustable speed lines and removes only
the F word from other moving lines. On a changed, unslowed value it emits
the original F spelling, including any fractional digits, but stores the
integer comparison value. This is a cooling output rule, not a change to the
decimal feedrate used for motion or duration calculations.

## Rust destination

`project_slice/gcode_emit/cooling/feedrate/rewrite.rs` already tracks the
integer value and preserves the original token. The pressure-marker path
currently keeps a synthetic repeated F word after that decision and relies
on `cooling/markers.rs` to deduplicate it later. That second modal tracker
ignores fractional F words. Make the cooling rewrite authoritative for
feedrate emission; the marker cleanup is only a compatibility layer for
removing internal tags, not a second modal feedrate pass. No change to
pressure adjustment, token precision, cooling time, fan control, or the
G-code processor is in scope.

## Verification

Slice a synthetic counterbore plate and a two-pier span on Prusa MK4 at
slope values 13 and 280. The plate must retain a speed command restoring
`F9240` after `F9251.231`; the span must suppress the redundant `F9540`
after `F9540.608`. Check the Native commands surrounding each transition
and compare the entire Native, public API, and CLI outputs with only the
generator line normalized. Include a K1 plate control with the equalizer
enabled. Use the released and seeded OrcaSlicer routes for the comparisons.
