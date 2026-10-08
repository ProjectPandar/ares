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
  from the phase RED. Missing-helper compilation is not that assertion RED.

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

## Gates before review

Rebase to latest Rust main (currently7a3216f67) and run fresh same-round
base/candidate seeded and official controls, recording exact failure names.
Run workspace nextest, clippy all targets/features -Dwarnings, fmt and wasm32.
All heavy checks are sequential,16 test threads, jobs1; browser windows defer
new heavy jobs. Focused earlier103/103 core and six Type2 complete-output
positive groups are useful local evidence, not final-main gates.
