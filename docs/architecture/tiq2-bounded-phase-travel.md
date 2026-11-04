# TiQ2 bounded CLI apply and support travel rewrite

## Decision and Rust boundary

This slice ports the CLI Print::apply lifecycle and regionless SupportLayer
avoid-crossing inputs. It does not claim complete TiQ2 G-code parity.

- `OrcaSlicer.cpp:6040` calls Print::apply once. `PrintApply.cpp:1620-1621`
  invokes normalize_fdm_2 after extruders(true); `PrintConfig.cpp:8688-8710`
  disables the tower when one filament is used. Rust effective_config/phases
  must retain cold and late normalization in that single apply. The second
  apply came from the GUI PartPlate scaffold, not this CLI boundary, and is
  deleted rather than guarded or retained as fallback.
- `GCode.cpp:5343-5429` initializes planning per instance, switches to
  SupportLayer, then restores the object layer without another init_layer.
  `AvoidCrossingPerimeters.cpp:480-507` uses a scaled-f32 nozzle average for
  layers without regions. Object-printing slots come from
  `PrintObject.cpp:3784-3803`, not all configured slots or only the selected
  support nozzle. `AvoidCrossingPerimeters.cpp:1100-1134,1252-1264` owns the
  support boundary and cache reuse. Rust layers/support_entry and
  motion/path/avoid_crossing pass this spacing directly in scaled units and
  expose no model Top surfaces on SupportLayer. Instance resets, model safe
  zones, Clipper, routing and coordinate rounding remain unchanged.

The effective-config orchestration remains a temporary compatibility shell
for the cited Print::apply concept. No independent Ares pipeline, ToolOrdering
condition, tower guard or legacy fallback is introduced. libvgcode is unchanged.

## Native conclusions

The source probes in tools/orca-probes/tiq2_{order,travel,travel_history}
were qualified against the complete seeded oracle, allowing only generated-by
banner normalization, before their traces were interpreted.

Native normalization has two calls: cold(objects0,used0,enable1) and
late(objects1,used1,enable0). The independent phase RED failed with four
calls instead of two; the single-apply fix passes that assertion.

For the captured travel, object initialization uses spacing437079. The first
internal cache construction occurs on SupportLayer with no regions, default
spacing400000 and below-layer square±5000000, producing square±4400000.
Object travel reuses that cache: intermediate point(4399900,4301461), plus
origin(165,165), emits X169.4 Y169.301. The independent real public-slice RED
failed with X169.344; the support-input fix passes. The scaled-nozzle kernel
also covers unequal active diameters and exclusion of inactive slots.

No Native-generated output is committed. The3mf is input, probes are source
instrumentation recipes, and generated streams/logs remain runtime artifacts.

## Included and deferred behavior

Included: single CLI apply, late normalization's one-filament tower disabling,
regionless support nozzle spacing, empty support Top surfaces and preservation
of the support→object cache lifetime.

Deferred: full TiQ2 output parity, independent skirt hull behavior, and adjacent
speed/extrusion behavior. After the travel fix, the observed TiQ2 streams agree
through1264 lines; first difference line1265 is Outer wall WIDTH0.38292,
Native F2700 versus Ares F2160. Subsequent micro-segment E values differ and
Native changes to F4560. Both streams have zero tower sections and footer
enable=1. This residual is not filtered or waived; the original whole-file
CLI RED is excluded from this bounded PR and remains for the next slice.

## Verification conclusion

Independent Native-assertion REDs precede each production fix. Latest directed
checks pass104/104. Final fresh full seeded on5d5384d92 passes685/685
on both base and candidate, with no skipped cases. After J1's Top-input
change, affected Type2 seeded groups were rerun and pass6/6. Latest workspace
passes8100/8100 with9 skipped; no tests were deleted. Clippy workspace/all-targets/
all-features-Dwarnings, fmt and wasm32 core+adapter pass. Checks use separate
targets, fresh caches and explicit Native producer/resources.

Official full controls have the same failure names on base and candidate.
A paired recheck has no timeouts: change_filament_flush_blocks passes
on both; narrowed_x2d_whole_file and bridge_over_infill_multi_filament_x2d
fail on both, with no candidate-only names. This establishes bounded
no-new-failure evidence, not stock or complete TiQ2 parity.
