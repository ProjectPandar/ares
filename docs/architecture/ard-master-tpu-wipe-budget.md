# ARD: Wipe budget and logical path reader parity

## Source and Rust boundary

Port the distinct readers in `OrcaSlicer/src/libslic3r/GCode.cpp`:
`Wipe::calculateWipeRetractionLengths` (:319, :342-351) reads physical
`extruder_id()` for budget distance/speed; `Wipe::wipe` (:451, :468-479)
uses logical filament `id()` for path clipping and dE distribution.

Rust destinations are `motion/options.rs`, its existing
`build/fields/retraction.rs` reader, and `motion/travel/wipe_path.rs`.
Reuse the existing `Prelude.physical` mapping from `filament_map`; no new
index resolver. Carry one private physical budget-distance scalar.
The existing logical `wipe_distance` remains the path/distribution input.

## Non-negotiable decisions

- Preserve `wipe_sequence.rs` before/during/top-up/reset formulas.
- Preserve scaled path construction, clipping, XY rounding and distribution.
- Keep ordinary writer retraction length/feedrate on their existing readers.
- Only the during-wipe calculation speed/distance use the physical reader.
- No guards, fallback, probes, native process code in core, or tolerance changes.
- Fan, refresh and nullable export remain separately owned changes.

## Test-first evidence and verification

The unchanged public validated API/real CLI whole-output A2 test was RED at
`2f2dedf3f`: Native PLA wipe E-.4 versus Ares pre-.11429/during-.28571;
Native TPU pre-.09298/during-.70702 versus Ares during-.8.
Raw Native/input/commands and both outputs are retained externally.

Failure modes are conflating the indices, altering logical path coordinates,
or applying physical selection to the ordinary writer retraction fields.
The existing clipping test receives an explicit identical 1mm budget input;
its assertions are unchanged and no unit test is added.

Engineering gates are not runtime parity. Current fan2 panic must first be
replaced by the consolidated fan commit; then run unchanged A2 and
retain its first difference. After upward integration/rebase, verify original
A1/A2, workspace/strict/WASM and seeded/official full pairs.
Single-tool/identity-map common outputs must remain byte-identical.
