# SPEC: `GCodeWriter::set_jerk_xy` approximate-value deduplication

Status: discovery / SPEC-RR only. Base `9cfaf0d16f0f5c4bf5fb6bf6c62d1c54e0d94ecf`.
No production, test or committed-input changes accompany this proposal.

## Upstream boundary and Rust destination

This is a source-cited writer rewrite correction, not an Ares jerk policy.

- `OrcaSlicer/src/libslic3r/libslic3r.h:52`: `EPSILON = 1e-4`.
- `libslic3r.h:306-309`, `is_approx`: strict
  `fabs(double(value) - double(test_value)) < double(precision)`, default EPSILON.
- `GCodeWriter.cpp:255-260`, `GCodeWriter::set_jerk_xy`: return without emitting
  or updating `m_last_jerk` if jerk is below0.01 or approximately the last value.
  The cache assignment follows that return and stores the requested value.
- `GCodeWriter.cpp:263-307`: flavor formatting and machine-axis clamps run only
  after admission. They are not the owner of approximate deduplication.

Rust destination is ONLY
`crates/ares-core/src/project_slice/gcode_emit/motion/jerk.rs:3-13`, `set`.
Its early cached-value comparison currently uses `< 1.0e-6`, so requests whose
change lies between the two tolerances generate extra commands. The surrounding
cache-before-clamp structure already mirrors upstream and must remain intact.
The current helper is a compatibility shell for `set_jerk_xy`, not a separate
Ares-owned tolerance specification.

## Reproducible difference

Stock `tests/parity/orca_cli_ender3.3mf`, tower disabled in every case. Create a
common control with `default_jerk`, `outer_wall_jerk`, `inner_wall_jerk`,
`top_surface_jerk`, `infill_jerk`, `initial_layer_jerk`, `travel_jerk` all7, and
`initial_layer_travel_jerk=100%`. The target changes ONLY `inner_wall_jerk` to
7.00005. This stays below both stock XY machine limits8, avoiding clamp ambiguity.
Geometry, templates, temperature, lift height and all other options unchanged.

Schema owner: `PrintConfig.cpp:3317-3324`, `inner_wall_jerk` is `coFloat`, min0,
with no configured quantization step. GUI registration `slic3r/GUI/Tab.cpp:2566-2568`
(Speed → Jerk XY → Inner wall). These are accepted saved floating settings;
this numerical regression does not claim the GUI displays five decimal places
or recommend that degree of physical tuning.

Fresh serial official Native double replay and actual Ares CLI:

| Input | Inner jerk | SHA256 | Whole generator-normalized equality |
|---|---|---|---|
| stock-control | original profile | `a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae` | PASS |
| equal-control | 7 | `afe37155ecbdb81ebe72e49a8e5dd5709b359f0e63f73d0e50a3689f12d2fe2b` | PASS |
| within-native-tolerance | 7.00005 | `e7a5ca34d4981a559d82d6895f59f5a642ddbdb69e49aa715aef5bd5eb351a80` | FAIL |
| above-native-tolerance | 7.0002 | `64abf2e38a99dfe87d84d3380149794bfd37779bf6d3d8821445e22bb0c66847` | PASS |
| within-both-tolerances | 7.0000001 | `335b65c5f33a7ca0a71f72b926c325965179ef3e6eca912a7bbb341f382f33ff` | PASS |

All10 fresh Native slices replay-stable, all subprocess exits0. Target first
normalized difference byte6179 / line272: Native begins `;TYPE:Inner wall`, Ares
inserts `M205 X7.00005 Y7.00005`. Ares also restores `M205 X7 Y7` before travel
out of the inner wall. Target has49 such extra changed-value commands plus49
restorations: Native2 versus Ares100 total M205 records. These counts are
observations for diagnosis, not future hardcoded output goldens.

This is full ordered-byte discovery; no M73/config/footer filtering is applied.
The within-both control also proves the current Ares dedup path is reached.
The above-native control proves that distinct admitted jerk commands should
still be emitted, not suppressed unconditionally.

## Included and deferred upstream behavior

After separate approval, port the exact default-precision `is_approx` admission
rule in `set_jerk_xy`: strict absolute difference below1e-4; preserve the0.01
lower gate and do not update the cache on a suppressed request. Do not compare
formatted text, rounded/clamped jerk or accumulate suppressed deltas into cache.
Use the source-owned tolerance; no formatter-precision or invented relative test.

Deferred: `GCodeWriter::set_accel_and_jerk` (`GCodeWriter.cpp:311-350`, Klipper
merged path), acceleration, jerk serialization, machine clamp correctness,
processor word parsing/time algorithms, pressure advance, templates/bed/exclude
name, fan/fuzzy/tower/skirt/brim/support speed, and every reserved worker slice.
No changes to the frozen W4 temperature or lift chains/const/state/approach files.
The fact that the source tolerance has the same numerical value as lift EPSILON
does not authorize editing that frozen area. If shared-constant access requires
coordination, handle it only after approval, not by rewriting frozen commits.

## Proposed RED and exits (not run)

After SPEC approval: SHA-pinned Native-first public API+CLI RED for the target
and equal/above/within-both controls. Use unchanged `native_stream_case::check`:
fresh Native replay and semantic callback before complete unmasked strict stream
comparison. Official then seeded, CPU17/PIN17/jobs1/threads1. No Native outputs,
coordinate constants, comparison masks or helper changes committed.

Failures to pin before implementation: too-small/relative/rounded tolerance;
cache updating despite suppression; suppressing real >EPSILON changes; comparing
post-clamp rather than requested values. Do not assert decimal7.0001 as an exact
binary threshold: subtraction from7 has floating rounding. Any exact-edge test
must respect upstream binary arithmetic rather than inventing decimal equality.

Freeze RED-RR, then request separate GREEN authorization. Proposed eventual exit:
all directed full streams pass both modes, fmt and workspace all-target clippy
pass, plus only coordinator-authorized integration-tip FULL. This document makes
no API, seeded, test-RED/GREEN or FULL acceptance claim for this new slice.

## Evidence and ownership

`${EVIDENCE_ROOT}/w4-jerk-dedup-tolerance/`: `probe.py`, `probe.log`,
`results.json`, five archives and per-case Native double streams/logs, actual
Ares stream/log, complete `ordered.diff`, `overlap-audit.json`,
`binary-provenance.json`. CPU17 serial, `ARES_PARITY_THREADS=1`,
`ARES_PIN_OFFSET=17`, `OMP_NUM_THREADS=1`, `RAYON_NUM_THREADS=1`, official inherited
CPU wrapper with setarch. No builds were needed for discovery.

Existing discovery CLI `/tmp/w4-temperature-red-target/debug/ares` built at
`a5cc569c05857121af80533679c801f6258b9cb5`; entire motion subtree tree-identical to
SPEC base main9cfaf0d16. Frozen lift is not in that binary; stock0.4 lift and
spaced start temperatures isolate this candidate from both frozen slices.
No exact-main rebuild qualification is implied.

Preproposal audit: `motion/jerk.rs` unchanged in W5 Type1 work/proposal, W1 fuzzy
qualified/startup fan, W6 frozen six-chain, W2 skirt/support/seam scope and frozen
515e95975, and both W4 frozen chains. A328-ref worker1/2/3/6 + W5 architecture
scan found no jerk tolerance/dedup/is_approx/epsilon SPEC reservation. Exact tips,
merge bases and overlap results retained in evidence. W3 processor-word owner is
explicitly excluded: this proposal changes writer-command admission only.

Stop after SPEC-RR; await approval before adding public RED or implementation.
