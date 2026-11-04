# SPEC: machine XY jerk writer cache rounding

No FULL. No raw config/header/Z/E/acceleration changes.

## Non-negotiable upstream rewrite boundary

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:32-35::apply_print_config` enables
machine limits for MarlinLegacy, MarlinFirmware, Klipper and RepRapFirmware.
`:53-56` caches `m_max_jerk_x/y = std::lrint(front)` only under that gate;
otherwise constructor-zero caches remain0. Under default FE_TONEAREST,
`f64::round_ties_even` matches the conversion. Do not use half-away `.round()`.
Caches are double members, so keep Rust f64 representation after rounding.
Consumers `set_jerk_xy:289-293` and merged `set_accel_and_jerk` use those caches.

Rust destination ONLY `gcode_emit/motion/options/build/fields/assemble.rs`,
max_jerk_x/y construction (currently raw first_float). Add source-owned flavor
predicate and convert just these two caches, falsegate0. Preserve all other
fields. Existing MotionOptions assembly is a shell for upstream writer caches,
not permission for a new config-normalization or machine-limit policy.
Included: same flavor admission plus nearest/ties-even conversion of XY limits.
Deferred: header/raw optionserialization, Z/E limits, acceleration, consumers,
jerk admission/cache/formatting/comments, templates/processor/all other owners.
No helper/API/lint suppression or changes to frozen writer-slice commits.

## Witness / GUI provenance

Common stockEnder3/toweroffMarlin2/default_jerk9 enables actual bodyM205.
Sole-key target machine_max_jerk_xvector8 ->7.6, bothnormal/silententries. GUI
machine motion jerk fields `Tab.cpp:5634-5635`. Valid Nativeaccepted saved
floating value, no displayedprecision or physicaltuningclaim.
Native bodyM205X8Y8 versus AresX7.6Y8, firstnormalizeddifferencebyte1405;
raw header7.60 remains equal. Integercontrol fullPASS. An inactive control with
stockdefault_jerk0 andXlimit7.6 also fullPASS because no bodyjerk commands emit.
No geometry/template/otherjerk/temp/lift/PA changes.

SHA:
- activeintegercontrol:fc0fba0a17069c48a0ddcd6bd332025db4702f659d71cbd9c6a7f5a087990132
- active7.6target:4ea82ec44a0d1c7527800e666c7a294c71b717fb90a1c361d09f82dff00241d7
- inactive7.6control:6b3c9d972c96f04cc849e3da9d67ff06679146d94b53e39819fb16916be4d152

Discovery: 8 Native slices over 4 initial/active cases stable, all CLI exit 0.

## RED-first failure modes / exits

Three SHA-pinned cases using direct existing native_stream_case calls.
Native replay precedes callback: parse actualinputlimits and headerM205 to prove
rawlimitsretain precision; activebodyM205 must obey roundedcache limits, inactive
control must remainwithoutbodyjerk. Then wholeunmaskedAPI/CLIcomparisonbothmodes,
no Nativegoldens/coordinate/countconstants/newhelpers/outputmasks.

Failureclasses beforecode:rawinsteadroundedcache,half-awayties,unconditional
rounding withwrongflavorgate,roundingheader/Z/E/acceleration,changingconsumer
clamps. Publicinputs pin non-tie7.6 and gate-in behavior; source review must also
verify ties-even and falsegate0 explicitly, not infer from these bytes.

Official then seeded: only target expected FAIL,
controlsPASS/Nativecallback+replay+API==CLIqualify. Commit REDbeforeproduction.
GREEN onlyXYcacheconstruction; freezeREDtests/inputs. fmt/workspaceall-target
clippy/3 cases both modes PASS. Group registration
withoutsqueeze,touchedRust≤400LOC.
