# SPEC: fixed pressure-advance signed-zero serialization

## Upstream source owner and Rust destination

This is a small source-cited serialization rewrite slice, not a new PA policy.

- `OrcaSlicer/src/libslic3r/GCode.cpp:3155-3161`: the initial-filament fixed PA
  call after filament-start G-code is the startup witness. The same writer call
  and adaptive-cache reset also occur at `GCode.cpp:8047-8052` for tool changes.
- `GCodeWriter.cpp:370-391`, `GCodeWriter::set_pressure_advance`: reject only
  `pa < 0`, then stream the admitted value with `std::setprecision(4)` and the
  normal stream defaultfloat behavior. IEEE negative zero is not less than zero
  and the streamed representation retains its sign.
- `GCodeWriter.cpp:389`: ordinary Marlin `M900 K` emission is the direct witness.

Exact Ares destination:
`crates/ares-core/src/project_slice/gcode_emit/pressure_advance.rs:59-62`,
private `format_pa`. Its `value == 0.0` branch unconditionally returns `"0"`,
collapsing negative zero. The command caller at37-42 already admits negative
zero just as upstream does. Option handling and embedded config serialization
preserve the sign in this witness; the only observed divergence is the emitted
fixed-PA command. The existing private formatter is a compatibility shell for
upstream ostream behavior, not authority to canonicalize valid signed inputs.

## Reproducible difference and scope of impact

Stock `tests/parity/orca_cli_ender3.3mf`, prime tower disabled in all cases.
Enable fixed PA, then compare `pressure_advance=["0"]` with `["-0"]`.
Only this real filament setting differs between those two controlled inputs.
No geometry, template, temperature, hop, jerk or other option changes.
Adaptive PA stays disabled; no template expression or processor-word input.

This is byte fidelity only: `K-0` and `K0` do not imply different physical
pressure-advance behavior. Do not present the defect as a safety or quality
problem. It is still a deterministic difference from source-defined output.

Schema: `PrintConfig.cpp:2340-2351`, enable boolean and floating PA vector,
max2/default0.02 with no signed-zero normalization or quantization rule here.
GUI registration `slic3r/GUI/Tab.cpp:3936-3941`, Flow ratio and Pressure Advance.
These are Native-accepted saved floating option values. No claim that the GUI
normalizes/preserves the displayed sign is needed or made.

Fresh official Native double replay and actual Ares CLI:

| Case | Fixed PA | SHA256 | Whole generator-normalized equality |
|---|---|---|---|
| stock-control | disabled | `a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae` | PASS |
| positive-zero | enabled0 | `67ffaf0ffe1409ebf2a264fe3b8b087b84ca61cc035db12a529be3077e55553b` | PASS |
| negative-zero | enabled-0 | `752f7ddf2c430856d90877a5e902100338fb9689ce62884cf7c9bec27c7e6fab` | FAIL |
| positive-small | enabled0.00001 | `106ec1f348b53932d8b544a39aa087f6c8832f462af234c954d29bb2f2b2f1bc` | PASS |
| rounding-edge | enabled0.000099995 | `7406814d931424820d5f571c9a9ebaaf820a0e5b22379b56f25ce7d80b7d82f5` | PASS |
| rounding-big | enabled0.99995 | `df2245d1fc38291fdcefaecee3106d43717c0c66f6980cf74b281a87c1387312` | PASS |

All12 fresh Native slices replay-stable, all subprocesses exit0. Target first
normalized difference byte1197, the minus sign in
`M900 K-0; Override pressure advance value` versus Ares `M900 K0; ...`.
The complete unified diff has only that one command-line change; config/footer
agree. No masks, formatting filters or Native-reference constants are used to
accept the result. Controls prove ordinary positive zero and four-significant
formatting remain intact on the same public CLI path.

## Included and deferred behavior

Preserve the sign of admitted IEEE zero in this private
fixed-PA number formatter, matching defaultfloat. Keep the negative-value guard,
nonzero significant-digit formatting, command syntax and writer side effects
unchanged. No new general formatter/API or numerical normalization policy.

Deferred: adaptive PA models/cache algorithms, root standalone G-code APIs,
other ostream edge cases, BBL/flavor precedence, machine configuration,
processor parsing/time/statistics, templates/bed/exclude-name, fan/fuzzy, tower,
and skirt/brim/support speed, and the temperature/lift/jerk production areas.
Do not alter input parsing or strip signed zero from config as a workaround.
If the actual public RED exposes another owner, stop before expanding scope.

## Proposed RED / exits

SHA-pinned Native-first public API+CLI target for signed
zero plus positive-zero, ordinary positive and disabled controls. Qualify two
fresh Native streams and the admitted PA command's observed sign before complete
unmasked generator-only comparison. Official then seeded.
No committed Native outputs or precomputed G-code goldens.

Failures to pin before implementation: zero-sign collapse; excluding negative
zero as if strictly negative; changing positive-zero serialization; changing
nonzero precision or swallowing the actual command. Internal IEEE sign must
reach output; public command observation must precede Ares comparison.

Exit: all directed complete streams pass both modes, fmt/workspace all-target
clippy pass, and the full parity suite passes.
