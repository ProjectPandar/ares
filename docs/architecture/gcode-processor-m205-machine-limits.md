# M205 processor machine-limit mutation rewrite

## Status and owning source boundary

Worker1, source-cited spec approved as `fa78fb34afa6df850742df777bff4d33e917cd23`.
Rebased before RED onto integrated base
`c4572b368a85e7c5dc445503b03880035b7358e3`; upstream revision
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Owner: `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:5236–5265`,
`GCodeProcessor::process_M205`, not emitter geometry, a firmware implementation
or an independently designed Ares planner. Rust owner:
`ares-core/src/project_slice/gcode_emit/processor/motion/config_commands.rs`.
The existing MotionState is a compatibility representation of the upstream
TimeMachine/machine-limit state; reuse it, do not introduce a second processor.

Included source mutations, in order:
- X writes both machine_max_jerk_x and machine_max_jerk_y (5241–5245).
- Explicit Y follows and overrides only Y (5248–5249); Z/E remain independent.
- S sets machine_min_extruding_rate (5256–5258), T sets
  machine_min_travel_rate (5260–5261), J sets machine_max_junction_deviation
  (5263–5264).
- Present numeric words use the existing f32 GCodeReader-equivalent parser.
  Missing/bare words retain state. No new clamping, scaling by units, zero
  replacement, selector policy, environment switch or printer-name special case.
- Mutations produce no motion block and do not insert/rewrite the command.
  Normal-mode processing is unconditional; no new independent Stealth-mode
  pipeline. Preserve the existing envelope/mode boundaries.

## Minimal consumer prerequisite — S/T must affect timing

Read-only source inspection shows current MotionState/ProcessorLimits carries
junction deviation but **no minimum-rate fields**. Merely accepting S/T tokens
would not port their behavior. Existing gcode_filament_stats.rs minimum-rate
arithmetic is a different scaffold, not the owning GCodeProcessor planner.

Include only this direct prerequisite in the same rewrite slice:
- `GCodeProcessor.cpp:2087–2098,2430–2436`: machine-limit initialization and RRF
  initial min-rate reset. Existing typed MachineEnvelopeOptions already owns
  both arrays; bridge Normal index0 via ProcessorLimits/envelope.rs and
  motion/limits.rs into private MotionState. No new public option or API field.
- `:3987–3989` and `:4347–4349`: at linear/arc motion-block creation, select
  minimum travel feedrate iff **E delta is exactly zero**; otherwise select
  minimum extruding feedrate, including retract/unretract and negative E.
- `:5726–5740`: feedrate floor is max(current raw feedrate, selected machine
  minimum). It affects the block, not persistent commanded feedrate; normal
  downstream per-axis/feedrate/acceleration limits remain authoritative.
- Both regular motion and arc segment builders need the same private state
  semantics, not separate floor policies. Preserve existing distance rounding,
  indexing, dropped moves, planner caching, axis limits and event ordering.
- J feeds existing junction-deviation/jerk consumers; do not redesign their
  formulas or widen this slice to unrelated JD/centripetal fixes.

Rust destination additions are bounded to ProcessorLimits, envelope.rs,
MotionState/limits.rs, existing linear/segment builders and M205 handler. Split
only if a touched Rust module would exceed400LOC. No file I/O/platform logic
in core; CLI/WASM keep consuming the same byte/options authority.

## Agreed public test seams and independent authority

Use the existing project byte/options slicing seam for machine_start/custom
G-code (`slice_project[_plate]_processed`/existing parity runner) and its public
processed result. Same admitted fixture/options go to identified Native CLI.
Compare **complete ordered output bytes**, excluding only generated-by header,
plus M73/footer values. No timings-only comparator or numerical adjustment.

Coordinator admission ruling: the six same-project full-byte comparisons are
the authority, including M73 and printed estimated times. Four business cases
are divergent; zero-rate and RRF configuration-reset controls match. The
standalone Native processor versus slice-inline processor per-vertex time-bit
diagnostic also failed the zero control (different initialization) and is
**not comparable**. It is retained only in `${TMP_ROOT}/ares-m205-c457/red`, excluded
from the RED commit and forbidden as a repair driver. No additional
PAUSE_PRINT marker/seam is admitted. No diagnostic rounding patch or relaxed
comparison is authorized.

## Test-first failure inventory and RED plan

Before production, independently commit test-only RED with immutable inputs,
commands, effective-option evidence, complete outputs and first differences.
Cover both admitted sources of minimum rates: nonzero configuration values for
machine_min_extruding_rate/machine_min_travel_rate without M205 S/T, and
machine_start M205 S/T/J/X mutations. Each source compares Native planner time,
M73 and complete ordered output bytes. Add control cases isolating the source;
do not let emitted M205 overwrite and conceal configuration initialization.
Start with one X-only custom command on asymmetric initial X/Y jerk; make real
Y/corner motions exercise inheritance. Then source-backed vertical cases for:
- X+Y: source-order Y override, independent Z/E preservation.
- S/T: deliberately low commanded feeds with both E-zero travel and E-nonzero
  extrusion/retraction, so floors cannot be inert; absent words retain state.
- J: Marlin firmware case differing from initial JD, with actual corner moves;
  include zero J restoring the existing classic-jerk path when appropriate.

Do not assume stock macros activate the RED: roadmap impact inventory found no
stock X-only/S/T mutations and stock J restates the initial value. Preserve
machine_start placement and ensure later startup/envelope setters do not erase
the tested command before the motions. No unsupported profile/UB input, synthetic
Native numerical expectation, manufactured failure, or fixture change to chase
GREEN. Retain invalid setup evidence if encountered.

Failure modes to cover: X updates only X; Y wins in wrong order; S/T tokens
accepted but unused; travel/extrude selected by role or positive E instead of
exact E delta; floors alter persistent F; command units/clamps differ; missing
words zero state; J ignored/reset incorrectly; arc segments bypass floors;
mode/init defaults drift; custom moves absent from M73 cache; start-gcode
setter overwritten; diagnostics from a different G-code/config/producer.

## GREEN, controls and exit

Separate spec, test-only RED and minimal production GREEN commits. No new
ignores, relaxed comparator, numerical patch, retries-until-success or legacy
fallback. Fix one admitted vertical behavior at a time, keep original RED
inputs/assertions and intermediate results.

Required gates on current main: focused original RED + controls; seeded and
fresh official full parity controls; explicit profile_entry controls;
`cargo nextest run --workspace`; strict all-target/all-feature Clippy and fmt;
wasm32 core/adapter check/release plus required WASM/API integration evidence.
Seeded must all PASS. Official acceptance uses exact failure-name equality
against a fresh matching base, with no automatic ND waiver; any exception needs
coordinator review and complete-byte evidence. Record the single-attempt oracle
policy explicitly. Honor browser quiet windows for new heavy work.

Compare unchanged common raw Ares streams; inventory any intentional new M205
outputs and baseline differences. Enumerate stock printers with nonzero minimum
rate configuration and retain old/new Ares raw output identities per printer.
Any changed stream requires a same-input new-Ares/Native complete-byte PASS,
not merely an expected-time explanation or relaxed comparison.
Do not bundle worker3's C1/Sermoon, SLA,
Native crashes, Top Only reference sensitivity or Option-domain range work.

When verified/approved, update the roadmap's existing low-priority process_M205
paragraph to integrated status with source/commit/evidence links, without
confusing it with historical milestone M205 (line-width registry). Exact-hash
review, approval, fresh main verification, nonforce FF and independent remote
confirmation precede integration. The224 domain follow-up starts afterward.
