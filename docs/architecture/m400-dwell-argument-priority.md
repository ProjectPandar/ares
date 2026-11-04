# M400 S/P argument-priority rewrite slice

## Source ownership and Rust boundary

This ports libslic3r behavior, not an independently designed timing pipeline.

- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:5347–5355::process_M400`
  initializes float value_s/value_p to zero, then evaluates
  `has_value('S', value_s) || has_value('P', value_p)`. Valid S short-circuits:
  P is not read. Only when either lookup succeeds does it call
  `simulate_st_synchronize(value_s + value_p * .001)`.
- `GCodeProcessor.cpp:6011–6014::simulate_st_synchronize` forwards additional
  time to existing calculate_time; leave this scheduler behavior unchanged.
- `GCodeProcessor.cpp:4848–4856::process_G4` uses the same operand precedence.
- Exact Rust destination: existing
  `project_slice/gcode_emit/processor/delays.rs::command_delay`, M400 branch
  only (lines8–10 on base). It currently adds parsed S and P*.001, returning65
  for `M400 S5 P60000`, while Native uses5. Replace this compatibility shell's
  branch with the same valid-S-first, otherwise valid-P-milliseconds selection
  already used by its G4 branch. No second delay engine or generic parser change.

Included: M400 operand precedence and its no-argument return consistent with the
existing G4-shaped branch. Preserve current float word parsing and unit meanings.
Deferred: independent P-only numeric rounding refinements, generic parser/sign
handling, planner/flush scheduling, accumulation, delay attribution, cache IDs,
M73 percentages/formatting, G4 behavior, and all other slicing/material boundaries.
No new option, public API, crate, timing multiplier, guard or legacy fallback.

## No-S/P audit: no demonstrated synchronization defect

Upstream does NOT call simulate_st_synchronize when neither S nor P parses.
Rust `command_delay("M400")` currently returns Some(0.0). That scalar alone is
NOT a behavioral witness: `estimate.rs:123–136::Estimate::record` consumes the
returned delay only inside `if synchronizes_planner(code, measured_g29)`.
`estimate/commands.rs:65–75::synchronizes_planner` requires a valid S OR P for
M400. Bare M400 therefore adds NO FlushEvent and NO PendingDelay, matching
upstream's lack of synchronization. No separate consumer of command_delay exists
in production. The returned Some(0.0) is not enough to claim a public parity gap.

A G4-shaped M400 branch naturally returns None without arguments, but the
existing synchronization gate remains unchanged. No no-argument target is
admitted: no GUI-reachable Native/current-Ares divergent witness is established.
Independent no-argument synchronization work is deferred, not fabricated RED.

## Native-first evidence and GUI reachability

GUI field: machine_end_gcode (`slic3r/GUI/Tab.cpp:4611`). Three input-only Native
exports contain a fixed motion/marker bracket in that field. Same Classic cube,
single PLA, no brim, scarf, spiral, ironing or prime tower. No reserved brim/EFC/
material, combine-solid/Arachne speed/height, concentric ironing, Voronoi or
spiral seam gating boundary is involved.

| Role | Command | Native-exported input-only SHA256 |
|---|---|---|
| RED target | M400 S5 P60000 | `27e1d3ec0b3276a37ffe8bf7e9d8e3693fa9ccc0a7167f1a1fe8314db2de2d7e` |
| Sole-variable PASS control | M400 S5 | `827b8c6bf7451a18f5ebc47e3272291b3eadb5490d69de199cf041f9a20be442` |
| Source-neighbor PASS control | G4 S5 P60000 | `c8047d059449bc8ab069898c2849f98d3f4c8606cd6578fd09e04ec9345c7c37` |

Target/control differ only in Metadata/project_settings.config and only its
machine_end_gcode key, by the P60000 token. Geometry and all other payloads match.
No manual zip edits or Native output may be committed.

Read-only discovery on an unmodified d592 Git-archive build verified4185
Rust/Cargo blobs. All12 Native runs (three inputs × two modes × two repeats)
preceded Ares execution and were stable. Target CLI whole-stream FAIL in both
modes: first M73 P0 R24 versus P0 R25, Native24m45s/Ares25m45s. Both controls
completely PASS, without masking M73 or other metadata.

Read-only v2 GDB proof: Native synchronizer entry receives5.0, G1 ID13073,
caller process_M400; Rust normal return for exactly the target command is
Option<f64>::Some(65). Normal exits, may-call-functions off, no inferior calls/
register/data writes, complete outputs unchanged from non-GDB. Invalid v1
Native source5353 resolved5355/no sample and Rust empty-string-read probe are
excluded.

## Formal RED and exit contract

Freeze exactly the three existing Native exports above. Tests must directly call
`runner::native_stream_case::check`, which already delegates to canonical main
`native_api_cli.rs::NativeCase`; no copied helper or normalizer. Check runner LOC
before registration and keep every changed text file <400 lines.

At the exact committed RED hash, run official/seeded targeted three-case sets.
Require Native2 stable BEFORE public byte API and real CLI. Preserve
complete ordered generator-only Native/API and Native/CLI comparisons plus raw
API/CLI equality and successful runtime statuses. Never filter M73/time/motion.
The target must genuinely fail and both controls completely pass. Audit raw
artifacts independently on RED because NativeCase panics before its final raw
assertion on a Native mismatch; do not claim that unreachable assertion ran.

Sequence: docs-only SPEC -> test/input-only RED ->
production-only GREEN, limited to delays.rs M400 branch -> same
formal matrix and neighboring dwell/SYNC/time regressions. Any strict residual
means STOP.
