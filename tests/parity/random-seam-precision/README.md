# Staggered seam f32 interpolation witness

`seam_position: "random"` + `staggered_inner_seams: "1"` on the n=72 box
(sparse_infill_density 24.5%, layer_height 0.2). Witness for
[2026-10-20-random-seam-float-semantics.md](../../../docs/architecture/2026-10-20-random-seam-float-semantics.md)
(approved boundary c3c30003d, re-established as 47dbb8474).

Divergence: exactly one line differs, `G1 X100.675 Y118.143 E.0392`
(native) versus `E.03921` (Ares). Read-only GDB inspection established
that the target random final_seam_position is bit-identical; the first
observed unequal stage is downstream stagger interpolation
(`SeamPlacer.cpp:1604-1619` → `placement.rs::staggered_seam`). The
original attribution to random.rs ratio arithmetic was falsified.
See the revised SPEC for the intermediate-value table and scope.

Manifest freezes the whole inputs and every ZIP-entry SHA. Formal tests
reuse main's `runner/native_api_cli.rs::NativeCase`, without the borrowed
`ARES_PARITY_PROFILE_ENTRY_REFERENCE` gate or any duplicate runner.
Native availability is the only environment condition, as in main's
NativeCase users; required-Native runs cannot silently skip. Acceptance
removes only the existing generator identity/date line. Input, Native,
API, CLI and comparison.json artifacts are retained under the configured
artifact root. Targeted filter: `test(staggered_seam_interpolation_)`.
No pre-window FULL or integration evidence is claimed.

`random-control.3mf` changes ONLY `staggered_inner_seams` from "1" to
"0"; every other ZIP payload is identical. Both tests slice fresh with
Native, then the public validated API + bare CLI, and require strict
byte equality. Before the fix, control must PASS and the staggered
witness must FAIL on its single E line in official and seeded modes;
after the fix, both must PASS. The positive additionally pins Native's
`G1 X100.675 Y118.143 E.0392` premise.
