# SPEC: custom start G-code temperature-command scanning

Date: 2026-10-24 (UTC+08).

## 1. Rewrite owner and scope

One source-cited `libslic3r/GCode.cpp` rewrite slice: recognize temperature
commands in custom start G-code using the upstream character scanner rather
than assuming the command and arguments are whitespace-separated words.
This is not a new G-code grammar, temperature policy or slicing pipeline.

Upstream files under `OrcaSlicer/src/libslic3r`:

- `GCode.cpp:528-589::custom_gcode_sets_temperature`: scan each line, skip
  literal spaces/tabs, accept uppercase M (or G only when `include_g10`), parse
  the numeric command prefix with `strtol`, then scan parameters until comment
  or line terminator. Matching M commands set the presence flag even without
  a parseable S value. G10 only counts after successful S integer-prefix parse.
  Parameters may immediately follow the command number, e.g. `M140S60`.
  Preserve source integer-prefix semantics, whitespace rules, case sensitivity,
  comments, line boundaries and scan progression; do not replace it with an
  independent GCodeReader interpretation or a broad regex heuristic.
- `GCode.cpp:4025-4054::_print_first_layer_bed_temperature`: scan M140/M190;
  always update writer bed state, but emit generated bed heating only when
  the custom template does not already control it.
- `GCode.cpp:4058-4090::_print_first_layer_extruder_temperatures`: scan
  M104/M109 (and G10 for RRF), update writer state and suppress generated
  nozzle heating if already present.
- `GCode.cpp:3132` uses the same helper for M141/M191 chamber presence.
  The shared lexical operation should remain consistent for this existing
  caller, but this slice does not redesign chamber temperature policy.

Rust destination:
`crates/ares-core/src/project_slice/gcode_emit/machine/temperature.rs:212ff`,
private `sets_temperature`, plus only directly necessary local call adaptation.
It currently strips comments, trims a line, takes `split_ascii_whitespace()`'s
first word, and checks exact command strings. Thus `M140S60` is not recognized
as M140 even though the upstream scanner recognizes it. Its G10 path also
currently requires the S value as a separate parseable word.

The current boolean helper is a temporary compatibility shell around the
cited source scanner. Replace its lexical recognition semantics, not its
callers' heating policy or command formatters. Retain returned presence as
needed by existing callers; do not introduce persistent temperature state or
new user configuration. If source `temp_out` propagation exposes an additional
writer-state requirement beyond lexical suppression, report a separate boundary
rather than silently expanding this slice.

## 3. Exact-main reachable witness

Use stock Ender-3 `tests/parity/orca_cli_ender3.3mf`, disable prime tower, retain
its stock start template and change only separators between selected M commands
and their S parameters. Printer Settings → Machine G-code → machine start G-code
is a GUI editor (`slic3r/GUI/Tab.cpp:4593-4604`, option at4600). No new profile,
model geometry, private API or invalid command is needed.

Examples after the normal placeholder rendering:
`M140 S60` → `M140S60`, `M190 S60` → `M190S60`,
`M104 S220` → `M104S220`, `M109 S220` → `M109S220`.
The same patched archive is passed to both binaries. Existing stock comments,
homing, purging, output footer/config and motion remain part of the strict
comparison; no masks or Native stream constants are used.

Fresh discovery results on exact main `957e963c3`:

| Input | SHA256 | Native vs actual Ares CLI |
|---|---|---|
| spaced control | `a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae` | complete normalized bytes equal |
| compact bed only | `e78d5230a495fd486ee289496d1fd7ccd6f2e903a33efa3f3afb423a415890bf` | RED at byte585; Ares inserts M190 S60 |
| compact nozzle only | `9a41b0b7d33f19f322f45214dff1f182c4bd1479ed0419ab5e7da10f02280fed` | RED at byte585; Ares inserts M104 S220 |
| compact both | `aaa8ac9385cf717dfe4f527d71e47bccbb6dc4f49be4ca7a96df6790825f75ce` | RED at byte585; Ares inserts both |

Each input has two fresh official Native slices, identical in complete bytes
under generator-line normalization, BEFORE comparison to the existing exact-tip
Ares CLI executable. The Ares CLI executable was built at `957e963c3`; Native is official 2.4.2.

The compact-bed complete ordered diff is exactly one extra generated line
between machine limits and `;TYPE:Custom`:

```diff
 M205 X8.00 Y8.00 Z0.40 E5.00 ; sets the jerk limits, mm/sec
+M190 S60 ; set bed temperature and wait for it to be reached
 ;TYPE:Custom
```

Compact-nozzle analogously inserts only
`M104 S220 ; set nozzle temperature`. This is a source-located producer
mismatch before custom template output, not a motion/state or tower residual.
Current main source still has the whitespace-token check, so the missing scanner
has not already landed. Public API and seeded mode are NOT freshly exercised in
this docs-only phase; those are explicit RED requirements below.

## 4. Included / deferred and immutable boundaries

Included: source character-level recognition for existing M140/M190,
M104/M109/M141/M191 and include_g10-sensitive G10 callers, including adjacent
parameter syntax; preserve source handling of comments, spaces/tabs, CR/LF,
case, command-number and S integer prefixes. Keep existing automatically emitted
heating formats and ordering unchanged except suppression when upstream says
custom G-code owns heating. No new parser dependency is needed.

Deferred: new G-code syntax policy, templates/placeholder semantics, persistent
writer temperature tracking beyond the existing boolean seam, calibration,
chamber/exhaust policy, material temperatures, custom toolchanges, brim, wipe
and tower planning, state splits, geometry and GCodeProcessor/M73. Explicitly
**do not touch `gcode_emit/motion/state.rs`**, brim or wipe-tower files. Keep
all touched modules <400LOC. Preserve the existing helper unit tests; do not
modify or add isolated tests after production code. No legacy fallback/flags.

## 5. Proposed RED-first sequence

1. Add Native-first public API+CLI SHA-pinned test-only inputs via the existing
   `native_stream_case::check` seam. Targets: compact bed/nozzle, stock spaced
   control. Require fresh official then seeded Native replay and strict whole
   outputs; no masked comparison or expected Native file checked into git.
2. Qualify lexical boundaries using source-cited GUI start-template controls:
   matching command in comments must not suppress automatic heating; a genuine
   compact command must suppress it; include_g10=false must ignore G10 (do not
   mistake firmware retract for a temperature command). Add an RRF G10 S case
   only if a stock reachable profile produces a clean isolated witness.
3. Document invalid/malformed scanning behaviour only after observing Native;
   do not invent validation or rely on invalid input to demonstrate the gap.
4. Report RED exact tip before implementation. Then port only the private helper's scanner, preserve callers/state semantics,
   and run directed/neighbour gates.
