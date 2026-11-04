# OuterOnly contour/mask: admitted Native RED

SPEC `a68680255`.
Source: Brim.cpp:448-570,661-711,903-904 and ClipperUtils.hpp default Miter3.
No production edits precede this separate fixtures/tests/RED commit.

## Reproducible protocol and admission

Original control-set-v2 and fresh cache/artifacts per mode. Filter:
`test(outer_only_) | test(outer_inner_) | test(inner_only_) |
test(legacy_brim_) | test(brim_bed_)`, all features, success output immediate.
Seeded: 47 tests, **39 PASS / 8 FAIL**, exit100.
Official: 47 tests, **39 PASS / 8 FAIL**, exit100.
All eight failures are the new active OuterOnly cases. The 36 unchanged
combined/inner/N/O/E/bed controls pass, as do three new no-hole/inactive controls.
Strict test-target Clippy -D warnings and fmt pass.

All eleven inputs per mode are admitted by Native (return_code0), the validated
byte API and real CLI (exit0). API and CLI raw G-code is identical for all22
pairs. Every archive is input-only; regeneration is byte-stable. The generator
verifies closed, paired directed mesh edges and positive signed volume.

## Independent geometry witnesses

Counts below are positive first-layer Brim XY/E segments; both modes agree.

| Input | Native total / hole / detached | Ares total / hole / detached |
|---|---|---|
| rectangle | 349 / 0 / 0 | 572 / 223 / 0 |
| concave | 349 / 0 / 0 | 683 / 334 / 0 |
| two-holes | 88 / 0 / 0 | 187 / 99 / 0 |
| narrow-neck | 88 / 0 / 0 | 263 / 175 / 0 |
| positive-gap | 378 / 0 / 0 | 605 / 227 / 0 |
| collapsed-inset | 922 / 0 / 0 | 1102 / 180 / 0 |
| two-islands | 177 / 0 / 0 | 279 / 99 / 0 |
| clipped-contact | 226 / 0 / 0 | 280 / 0 / 52 |
| no-hole | 349 / 0 / 0 | 349 / 0 / 0 |
| zero-width / sub-pair-width | 0 / 0 / 0 each | 0 / 0 / 0 each |

The clipping input's printable notch occupies global y125..126.2 up to x130.
The model ends at y125, while the far-right corridor reconnects the printable
polygon beyond the brim. Native/API/CLI admission succeeds, and Native retains
exterior brim while removing the disconnected upper fragment. Thus this is an
active contact witness, not rejection, an empty selector or a gap-only guess.

First full-byte divergences are at byte451/line18, byte847/line32 or
byte988/line37, in normal timing/progress commands. The independent regional
witnesses establish the accompanying extra hole/detached geometry; no timing
normalization, assertion change or comparator relaxation is used. Complete
generator-only ordered comparison remains the acceptance check for both API
and CLI, and all raw streams are retained.

The seven hole cases expose the whole-ExPolygon OuterOnly scaffold; the eighth
exposes omitted final contact retention. These observed boundaries match the
specified replacement.
