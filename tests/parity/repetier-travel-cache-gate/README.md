# Repetier travel-acceleration cache admission inputs

Source: `GCodeWriter.cpp:49-52::apply_print_config`. The travel cache requires
both `use_mach_limits` and separate-travel support. Repetier supports M202 but
is excluded from machine-cache admission. The approved specification is
`docs/architecture/2026-10-24-repetier-travel-cache-gate-spec.md`.
Base: `33ed13702`. This tests-only RED does not change production code.

All cases derive from the genuine GUI-generated `orca_cli_ender3.3mf`, disable
the prime tower, and set `default_acceleration=1000`,
`initial_layer_acceleration=500`, and machine travel acceleration cap 9000.
Only flavor or travel request varies. No BBL model identity, geometry, template,
jerk, temperature, PA or fan changes are needed. These are admitted saved settings,
not factory firmware or physical tuning recommendations.

| Input | Request/flavor | SHA256 |
|---|---|---|
| repetier-over-limit.3mf | 10000/Repetier | a1bbe4c18e06b709f580a67fe8bec437ca1d080f1b70912075252229f0d53b65 |
| repetier-below-limit-control.3mf | 6000/Repetier | 97495393518915bbd5285bdd90785745f03b1ef50cc625d5251daeaf0fd64d07 |
| marlin2-limit-control.3mf | 10000/Marlin2 | 2b232b6735ba52645223c1adbf9734320efccbbd5f5f347df7458cb0c1317134 |

Three direct calls use the existing `native_stream_case` seam. Fresh Native
replay qualifies first. Callbacks read requested travel, cap and flavor from the
input and check the first actual body travel command. Repetier must retain its
M202 support and full request; Marlin2 must retain its machine-clamped M204 T.
The callback observes the body command, not the raw initial envelope. No hardcoded
Native coordinates, counts, reference output or Ares-helper calls are used.

Complete Native/API/CLI comparison follows with unchanged generator-line
normalization only. Timing, headers, configuration and footers remain included;
raw API==CLI is required. Callbacks are observations, not filters. Expected RED:
target fails at byte 1211 (`M202 X10000 Y10000` versus `M202 X9000 Y9000`), while
both controls pass. Discovery's target has exactly 50 M202 differences and equal
M73 records; the public runs must qualify independently. No Native output files
are committed.

Failure modes were documented before implementation: missing predicate
conjunction; disabling separate M202 instead of only the cache; disabling
Marlin2/RRF limits; changing rounding or print caches; inactive below-cap targets;
confusing raw envelope with body output; or hiding downstream differences.
The older BBL travel10000 archive is not included.

Freeze inputs/tests before GREEN. The authorized port only adds the existing
local `use_mach_limits` conjunction to the initializer in assemble.rs. Keep the
rounding, flavor set and zero branch; no helper or travel execution changes.
CPU17/PIN17, jobs=1 and threads=1; no FULL. Submit a combined SPEC/RED review
request and hold before production changes.
