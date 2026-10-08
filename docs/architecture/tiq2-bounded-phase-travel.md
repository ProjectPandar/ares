# TiQ2 bounded CLI apply and support travel rewrite

Status: independent REDs / bounded GREEN in progress. Coordinator approved
shipping phase+travel without the TiQ2 whole-file CLI RED; that RED remains
on `pi/type2-tower-activation` for the next outer-wall slice. No comparator
is weakened, no output is filtered, and complete TiQ2 parity is not claimed.

## Non-negotiable source boundaries

1. `OrcaSlicer.cpp:6040` calls Print::apply once. `PrintApply.cpp` performs
   cold normalization, then `:1620-1621` calls normalize_fdm_2 after
   extruders(true). `PrintConfig.cpp:8688-8710` disables the tower for one
   used filament. Remove Ares's GUI-derived synthetic second apply in
   effective_config/phases.rs; do not add a ToolOrdering/tower guard.
2. `GCode.cpp:5343-5429` initializes motion planning per instance, switches
   temporarily to SupportLayer, then restores object layer WITHOUT another
   init_layer. `AvoidCrossingPerimeters.cpp:480-507` falls back to the scaled
   f32 nozzle average when the active layer has no regions. Printing slots
   come from `PrintObject.cpp:3784-3803`, not all configured slots or just the
   selected support slot. `:1100-1134,1252-1264` owns support boundary
   construction and cache reuse. Rust destination: layers/support_entry,
   motion/path/avoid_crossing and its boundary builder. Preserve model safe
   zone and instance lifetime, pass the support fallback in scaled units,
   and do not change Clipper/router/waypoint rounding.

The existing pure effective-config orchestration is a temporary compatibility
shell for Print::apply. The historical second apply came from99fb0beba0's
Task19b3 GUI PartPlate plan, not a CLI whole-byte witness. It is removed,
not retained as fallback. Rendering-neutral libvgcode is unchanged.

## Native proof and RED obligations

Qualified seeded probes must equal the complete seeded-oracle output before
interpreting their traces (generated-by-only normalization). Recipes:
`tools/orca-probes/tiq2_order.nix`, `tiq2_travel.nix`,
`tiq2_travel_history.nix`; all derive from the exact seeded base/source.

- Normalization receipt `/tmp/tiq2-seeded-probe-20261007T210508Z/`:
  cold(objects0,used0,enable1) then late(objects1,used1,enable0); only two
  calls. Independent phase RED on aaa89f76a completed exit100 at the native
  call-count assertion, actual4/expected2:
  `/tmp/tiq2-bounded-phase-red-aaa.log`. It is not a compile failure.
- History receipt `/tmp/tiq2-history-qualified-20261008T0049Z/`:
  object init spacing437079; first internal construction occurs on support,
  regions0/default spacing400000, below lslices square±5000000. Cache is
  square±4400000. Object target reuses it, producing raw interior
  (4399900,4301461), origin165,165 ->world169.3999,169.301461. No wipe
  last_pos or post-retraction route change. Travel RED must assert this
  captured actual pipeline transition before the support fix, separately
  from the phase RED. Independent travel RED on7a3216f67 completed exit100,
  native assertion actual `G1 X169.344 Y169.301 Z.6` versus expected
  `G1 X169.4 Y169.301 Z.6`:
  `/tmp/tiq2-bounded-travel-red-7a.log`. Missing-helper compilation is not
  that assertion RED.

Failure modes: phantom apply resurrects enable; support inherits object flow
spacing; inactive configured nozzles or selected support nozzle determine
average; scaled f32 mean is truncated through mm; instance reset is removed
or support→object restoration resets the native cache.

## TiQ2 residual (not fixed or waived)

On aaa candidate with phase+support input correction, complete streams agree
through1264 lines. Next first difference at line1265, Outer wall WIDTH0.38292:
Native F2700 versus Ares F2160; following micro-segment E values also differ
and Native changes to F4560. Native/Ares normalized lengths816585/816194.
Artifacts `/tmp/tiq2-phase-support-aaa-seeded/case-5YcqzB`; log
`/tmp/tiq2-phase-support-aaa-seeded.log`. Both emit zero tower sections and
both footer enable=1. This residual is deferred to the next source-cited
CoolingBuffer/overhang-speed investigation, not normalized away.

## Candidate GREEN before final-main gates

Phase fix `ef71a684` passes102/102 focused cases on7a3216f67
(`/tmp/tiq2-bounded-phase-green-7a.log`). After restoring only the source
support input fix, the real pipeline travel assertion and scaled-nozzle
kernel pass alongside all phase/FDM checks:104/104,16 threads
(`/tmp/tiq2-bounded-travel-green-7a.log`). Each independent assertion RED
precedes its corresponding production change; the helper's additional
scaled-average test was written before implementation and is not presented
as the independent travel RED.

## Gate receipts and current requirements

All controls use fresh caches, explicit Native producer/resources, all reference
flags (including profile_entry), separate base/candidate targets,16 threads
and jobs1; heavy checks are sequential and deferred during browser windows.
Base worktree: `/tmp/ares-worker2-tiq2-gate-base`; candidate: this worktree.

-3cad8b4c3 full seeded: both649/649 passed.
-3cad8b4c3 full official: both638 passed/11 failed, same exact names;
  `/tmp/tiq2-bounded-full-3cad-official-exact-failures.json`. On2026-10-08
  at16:03 coordinator time, the official wrapper was fixed to inherit the
  pinned CPU instead of forcing every Native ontoCPU1. Pre-fix Native
  timeouts are **INCONCLUSIVE**, not accepted parity failures. The three
  timeout names require fresh paired verification with the fixed wrapper:
  bridge_over_infill_filament::orca_parity_bridge_over_infill_multi_filament_x2d,
  change_filament_flush_blocks::orca_parity_change_filament_flush_blocks,
  narrowed_x2d_whole_file::orca_parity_narrowed_x2d_whole_file.
-3cad8b4c3 workspace:8042/8042 passed,9 skipped;
  `/tmp/tiq2-bounded-workspace-3cad.log`.
-dbcd52d5a full fresh seeded: both660/660 passed, no skipped cases;
  `/tmp/tiq2-bounded-full-dbcd-{base,cand}-seeded.log`.
-1a8c92f55 clippy workspace/all-targets/all-features with-Dwarnings passed;
  `/tmp/tiq2-bounded-clippy-1a8.log`. Directed checks104/104 passed;
  `/tmp/tiq2-bounded-directed-1a8.log`.

Latest fresh full seeded ondc8d7f00c: base/candidate **668/668 PASS**,
no skipped cases; `/tmp/tiq2-bounded-full-dc8-{base,cand}-seeded.log`.
Coordinator permits reuse through3c1a79941 (only six Arachne narrow-neck
cases change), with refreshed clippy/directed. After J1's Top input change
on de57be8b0, affected fresh seeded Type2 groups **6/6 PASS** and directed
checks **104/104 PASS** were rerun, not carried across that boundary.

Post-fix official paired recheck on de57be8b0 completed without timeouts:
change_filament_flush_blocks passes on both; bridge_over_infill_filament
and narrowed_x2d_whole_file fail on both, with no candidate-only names.
`/tmp/tiq2-bounded-official-recheck-{base,cand}.log` and
`/tmp/tiq2-bounded-official-recheck-exact.json` replace the three old
INCONCLUSIVE timeout observations. The eight remaining full-control
failure names are identical in the earlier pair. Official outputs remain
non-identical to Ares; this is bounded no-new-failure evidence, not an
assertion of stock parity.

Final gates on ancestor b1f65d40d (range behavior):
clippy workspace/all-targets/all-features-Dwarnings, wasm32 core+wasm,
fmt, and directed **104/104 PASS**. Logs:
`/tmp/tiq2-bounded-clippy-final.log`, `/tmp/tiq2-bounded-wasm-final.log`,
`/tmp/tiq2-bounded-directed-final.log`. Coordinator permits evidence reuse
through ca1d32ea5 (role metadata; no G-code change) and7fc1c69e3 (by-object
clearance; no by-layer G-code change). Rebase before review/push.

No Native-generated output files are added by this slice: the committed
3mf is input, probes are source instrumentation recipes, and outputs and
logs stay in runtime cache/artifact directories. The captured source
assertions do not submit an oracle stream.

Earlier3b candidate results using a shared target incorrectly reused a base
binary with baked CARGO_MANIFEST_DIR. That runner-path assertion failure is
quarantined; all subsequent evidence uses independent targets. No failed or
inconclusive run is used to claim complete TiQ2 parity.

