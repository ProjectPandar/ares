# Arachne alternate extra wall: public Native RED

Production is unchanged from main `c22f5a81d`.
Input-only fixtures and public tests precede the guard removal.

## Run and admission

Each leg records commands, clean environment, input/test/binary hashes and
case results. The suite selects `test(arachne_alternate_)` with
RUSTUP_TOOLCHAIN=1.99.0; each leg has a fresh Native cache and artifact root.
This finite test calls the generic official/seeded slice producer directly;
it does not invoke the missing stable-options/dynamics processors.

Official: 10 paired tests, 2 PASS / 8 FAIL, 9.866 seconds, exit100.
Seeded: 10 paired tests, 2 PASS / 8 FAIL, 11.178 seconds, exit100.
40/40 Native slices return0 and the effective settings/wall witnesses pass.
Each leg's 10 off plus 2 inert on cases pass complete Native/API/CLI comparison.
Across both modes, all24 admitted Ares controls have identical API/CLI raw bytes;
48 control output streams are retained. The remaining8 on inputs per mode
return `UnsupportedProjectFeature("alternate_extra_wall")` from the validated
byte API and CLI exit1. No comparison is relaxed to turn rejection into PASS.

## Independent Native wall witnesses

Each number counts positive emitted wall XY/E crossings of y=110. Two crossings
represent one enclosing wall; the dual-island model has two independent pairs.

| Input pair | Off, layers 0–5 | On, layers 0–5 |
| --- | --- | --- |
| prism / density-one | 4,4,4,4,4,4 | 4,6,4,6,4,6 |
| density-zero / density-half | 4,4,4,4,4,4 | 4,4,4,4,4,4 |
| single-top / stepped-top | 4,4,4,4,4,2 | 4,6,4,6,4,2 |
| single-first | 2,4,4,4,4,4 | 2,6,4,6,4,6 |
| zero-walls | 0,0,0,0,0,0 | 0,2,0,2,0,2 |
| two-islands | 8,8,8,8,8,8 | 8,12,8,12,8,12 |

Spiral on/off preserves the full wall/infill witness, with seven emitted layer
blocks (six geometric layers plus the transition-out): each has2 crossings.
`PrintConfig.cpp:8567–8580` resets alternate extra wall to0 in both effective
spiral settings; `SpiralVase.cpp:123,153` owns the final constant-Z transition.
Input and effective normalization are asserted separately, not silently ignored.
The positive prism, zero-wall and two-island controls also emit sparse infill.

## Input preflight retained

The first uncommitted test draft is not the final RED gate.
Native showed that stepped layer4 is non-topmost and regains the full count
through `PerimeterGenerator.cpp:2240–2244`, and spiral includes transition-out
with normalized alternate0. Correct the draft's independent expectations to
these source-owned behaviors before committing any RED tests.
The final spiral input explicitly carries the normalized filament layer-retract
field; the final stepped case uses positive integer density1 instead of10.
This retains the alternate predicate/regeneration witness while its off case
passes full comparison; the earlier10% step's off fill/M73 differences remain
retained as preflight evidence, with no claim of a production repair here.

20 deterministic watertight input3MFs reproduce byte-for-byte. Each on/off pair
differs only in the alternate setting, with identical mesh/container members.
No Native G-code or output artifacts enter Git. Strict clippy for the test
target and rustfmt pass; broad engineering/coverage gates follow GREEN.
The obsolete negative unit assertion remains unchanged pending its explicit
[retirement record](arachne-alternate-extra-wall-test-retirement.md).
