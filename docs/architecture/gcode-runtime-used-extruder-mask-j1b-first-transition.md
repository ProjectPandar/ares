# J1b: first compatible toolchange template context

Status: SPEC-REVIEW requested. Expansion authorized by coordinator after
mask-only candidate exposed a later complete-byte residual. No J1b
implementation yet.

## Source boundary

Upstream pin8500fcdccaa10b5099ac20d252af3a7c560046f1:

- `GCode.cpp:2845–2859`: persistent parser gets initial logical
  `initial_tool`, `initial_extruder`, `current_extruder` and current hotend
  before machine startup. These are not defaults inferred from writer0.
- `GCode.cpp:7710–7747`, `GCodeWriter::need_toolchange/toolchange`: initial
  selection with no active writer filament is distinct from an actual
  transition away from a selected filament. Single/multiple branches remain.
- `GCode.cpp:7750–7758`: multiple-extruder call increments change count,
  retracts and resets wipe path; do not bypass the first call merely because
  the startup script already used the same tool.
- `GCode.cpp:7806–7858`: old filament comes from writer filament if present,
  otherwise `m_start_gcode_filament`; the startup scanner result participates
  in transition calculations but does not register an active writer filament.
- `GCode.cpp:7860–7898`: dynamic override provides previous/next extruder,
  hotends, layer and toolchange counters; it does NOT reset persistent
  `current_extruder` to writer0 before the first render.
- `GCode.cpp:7950–7980`: render custom change script when nonempty and not
  suppressed by first-call manual mode; append rendered text with upstream
  EOL rule, then `_FORCE_RESUME_FAN_SPEED` and position-clear handling.
- `GCode.cpp:7992–7997`: writer toolchange installs the new filament and
  its T command is suppressed only when the custom parsed text selects it.
- `GCode.cpp:8007–8015`: persistent current tool/hotend aliases update AFTER
  the change render, before filament-start template.
- `GCode.cpp:1663–1697,3894–3920`: writer refresh updates positions/E
  counters, not a fabricated initial `current_extruder`.
- `GCode.cpp:980`: separate wipe-tower toolchange fan marker source; not
  evidence to rewrite this J1 compatible startup path.
- `GCode/CoolingBuffer.cpp:543–545`: consumes the fan-resume marker as
  `TYPE_FORCE_RESUME_FAN`; do not change cooling parser/output policy.

## Evidence and corrected interpretation

Mask-only production commit `df56e12f0fe3fad0e071cbb717f830ff9dbd8ca6`
stands alone, NOT pushed and NOT declared whole-file GREEN. Parent is the
approved RED4f943a7ed. Mask now matches Native and the unused T0 flush is
removed. The unchanged primary inputSHA7d796432… still differs at
byte1845/line135: Native `_FORCE_RESUME_FAN_SPEED`, Ares blank followed by
`; Change T0 -> T1 (layer -1 1/0)`.

Frozen exact streams, input, manifest/error, usage witness and receipt:
`${TMP_ROOT}/ares-j1-mask/findings/primary-after-mask/FINDING.md` and
`finding.json`. Four fresh CPU0/seed0 Slice receipts,0TIMEOUT. Primary
remains DIVERGENT; three authorized witness-only controls remain PASS.

The stock J1 `change_filament_gcode` begins with its update comment, then
`{if current_extruder != next_extruder}` gates the Change/T/fan/quick-switch
block. Both parser's initial tool and new target should be1. Native retains
current1 and skips that conditional body, but still renders the nonempty
header and emits the fan-resume marker for the processed custom script.
Ares recreates base_config(current0) for the change render; its existing
writer refresh only supplies current tool when `extruder_selected` is true.
Before initial selection it is false, so current0 survives, incorrectly
entering the stock T0->T1 branch. Ares already emits the fan marker in
`tool_transition/change.rs`; adding another marker is not the repair.

`change::append` computes old from `start_gcode_filament` or selected writer,
then clears the startup scanner result; `dynamic::insert` populates
previous/next but not current. This loss of persistent initial parser state
is the source-owned compatibility-shell gap.

## Rust destination and minimal behavior

Destination: `gcode_emit/tool_transition/change.rs`, its dynamic context
module and/or `motion/writer_state.rs` / existing InitialFilaments metadata,
whichever installs the upstream initial current-tool value without changing
already-selected transitions. Choose the smallest connection at the first
change template context; no new public API or independent tool-ordering layer.

Included:
- Before initial compatible change render, preserve upstream initial logical
  current tool rather than base0. When writer already selected, keep its
  current logical tool unchanged. Do not replace current with next on every
  change; previous and current are distinct concepts.
- Keep stock template unchanged and its conditional selection meaningful.
- Retain first toolchange count, previous/next fields, manual-first suppression,
  startup last-tool scanning, writer installation, EOL, fan-resume marker,
  position clear and subsequent filament-start/temperature/PA behavior.
- Do not use mask contents, number of registered tools, physical ID or
  literal J1 printer name to invent a special startup rule.

Deferred: wipe-progress/M73 residual
`${TMP_ROOT}/ares-j1-mask/findings/wipe-progress/FINDING.md`; sequential ordering
lifecycle; generalized persistent custom user-variable state; template
expression changes, numerical/time/cooling fixes, geometry, seam and
unrelated toolchange policies. No marker stripping or injected output.

## TDD/review gates

The mask-only primary complete-byte RED is an independent pre-J1b witness;
retain it and its exact input/Native hashes. Request SPEC-REVIEW before
implementation. J1b production change must be a separate commit from
mask-only df56e12f. Existing primary assertion stays complete bytes with
only generated-by normalized; M73/footer/statistics retained. Three control
assertions remain exact witness-only under the prior ruling, with their
raw divergences and findings preserved.

After approval, test the smallest context repair using fresh CPU0/seed0
references, runs1, fresh caches, shared300s per-case deadlines and prebuild
outside deadline. Assert primary complete bytes, initial1, usage false/true,
and exact Native tool/flush sequence. Preserve ordinary role1, both-used
and by-object witnesses; add or reuse public-output coverage showing an
already-selected real change still renders old->new, rather than applying
initial=next indiscriminately.

If primary reveals another residual, stop and report exact bytes without
relaxing it. Once primary is complete-byte GREEN, run fresh base/candidate
full seeded, directed cases, strict Clippy-Dwarnings, rustfmt and WASM.
Official/full-workspace scope remains per latest coordinator ruling, with
any carried-forward evidence explicitly labeled. Candidate named failure
set must be subset of base. Audit RED/GREEN input/Native invariance and
by-object witness/output preservation; do not claim witness PASS is full
parity. Submit combined exact hashes in REVIEW-REQUEST, then only approved
nonforceFF push after latest-main rebase and independent remote confirmation.
