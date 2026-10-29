# Honeycomb anchoring ZAA inputs

Input-only 3MFs for approved SPEC `224fc3dc7`.
Upstream: `Fill.cpp:1440-1447` regional conjunction →
`FillBase.cpp:311-313` direction suppression → `FillHoneycomb.hpp:53`.

The mesh/profile bundle is the approved authored Honeycomb multiline target
(`tests/parity/honeycomb-multiline/honeycomb-multiline-target.stl`, 10mm high).
Only `zaa_enabled`, `zaa_dont_alternate_fill_direction`, and
`top_shell_layers` differ. All other ZIP members and metadata are preserved;
all files have nine input members and no G-code or preview output.

Common settings: multiline2, direction45, sparse20%, layer height0.2,
ZAA min_z0.05 and perimeter-height35. Public tests verify loaded options,
Native effective footer, positive Internal Bridge activity, unchanged Sparse
motion, two fresh Native repeats, and public API/CLI raw equality before
unchanged strict whole-file comparison. M73 is never removed from that
comparison. The target is expected to fail until the regional anchoring bit
is ported; all controls must remain exact.

The 50-layer model begins top shells at physical layer `51 - count`.
Thus top6 exercises anchoring object index43 (%3=1), top7 index42 (%3=0),
and top8 index41 (%3=2). No mesh search or pattern widening is involved.

| File | Role | SHA256 |
|---|---|---|
| top6-enabled-flag1.3mf | target | 078df2b3f6ffa92167b8c044408b02beaf5534a7bcebdbe6bebc606b074a9812 |
| top6-enabled-flag0.3mf | conjunction control | c1a68d0452738de551592b6bebdce8db631379320c3d25ea6f74c4c4a54f66f5 |
| top6-disabled-flag1.3mf | conjunction control | 662a314cc90f2a83c3a79de9e4c742b7bc29cd6eea30975bded1e1037079fbd7 |
| top7-enabled-flag1.3mf | phase-zero control | 6604daa38d5c7dd73f39311c7094e988e0b87317968ceab9d817650497d669df |
| top8-enabled-flag1.3mf | phase-two control | a838438b5bd0b7c5895ef5b167f77bee0cce1519a7babe3425bc6ad587e12753 |

Official/seeded discovery: each identical SHA has stable repeated and
cross-mode normalized Native bytes. All conjunction/phase controls passed;
only the top6 enabled/flag1 target diverged, with its first full-stream
M73 difference retained. No Native outputs are committed.

The sibling Line anchoring literal false (`arms.rs:216`) is explicitly
deferred in the SPEC; this input/test chain does not change or cover Line.
