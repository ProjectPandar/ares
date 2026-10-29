# SPEC: separate Klipper set_jerk_xy XY scalar clamp

Base13e6964485185d1d0f62e5a1f00b06933fb0d35d. CoordinatorACK authorizes SPEC →
Native-first public RED → Klipper-arm-only GREEN → review/hold. NoFULL.

## Source-owned boundary

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:263-270::set_jerk_xy` Klipper clamps
positive m_max_jerk_x THENpositive m_max_jerk_y before emitting one scalar.
Raw-request suppression257/cacheassignment260 precede clamps; retain that order.
Rustdestination ONLY `gcode_emit/motion/jerk.rs` Klipper emitted value, currently
X-clampedx. Existing clamp_xy(state,jerk) implements sourceXYsequence for merged
caller; reuse unchanged. Existing Ares shell must follow upstream writer, not a
new safetyfallback/policy. Included onlyKlipperarmvalue; no newhelper.
Deferred: sharedclampimplementation,cache/tolerance/limits/formatting/comments,
mergedwriter,Repetierotherflavors,processor/template/fan/otherowners. Coordinator
noted sameRepetierdefectclass asNEXTproposalonly; doNOTfoldin.

## Witness and reachability

StockEnder3/toweroffKlipper/commondefaultjerk7/initialjerk7,stockXY8limits.
Sole-keytargetY6;controlXY8and symmetricX6. GUI Tab5634-5635 machineXYjerk;
realintegervalues, no rounding/precisionconfound or UI/physicaltuningclaim.
Targetfirstdiff1232Native standaloneSCV6vsAres7,2layer-boundarylines only;
followingmergedcommandcorrect6bothsides. ControlsfullPASS,6freshNative slices
stable/allCLI0. Exact13eCLI discovery; evidence
`/home/indexyz/ares-evidence/w4-klipper-separate-jerk-xy-clamp/PROPOSAL.md`.
No API/seeded/RED/FULLqualification inferred.
SHAequal3a8b3f3fbbcbdc9b7b5eb1ee13e5e08533ae8e54d60fd814be46ff45b1c02114;
Ytarget744139aab80a6f3ead834cdd93c47190bfab2e717c1e8ecd7a4da47e306eb542;
Xcontrol289bbdd4c40fc49f1c5eafcceaa890de41ddb01d19fe66a2b1699a6ea7397032.

## RED-first failures / exit

3frozenSHAinputs/directexistingnative_stream_casecalls,nohelper. Nativefresh
replaytheninput-derivedmin-positiveXYcallback on actualFIRSTstandaloneSCVword
(excludemergedACCEL);allcasesrequest7abovelower6. NoNativecount/coordinate/value
goldens. CompleteunmaskedAPI/CLIcomparison/generatorlineonly/alloutputincluded.
Failureclasses:Yignored,positive-limitsemantics,cacheclampedinsteadofraw,
wrongmergedpath/otherflavorchange,formatterregression. Reviewsourcecacheorder;
publiccases do not prove everyprivatecachetransition/nonpositivecase.
CommitREDbeforeproduction,onlytargetFAIL/twocontrolsPASSbothmodes/Nativequal+
API==CLI. GREENONLYexistingclamp_xy reuse inKlipperarm,tests/inputsfrozen.
CPU17PIN17jobs1threads1officialthenseeded/fmt/workspacealltargetclippy/3cases
bothPASS,REVIEW-REQUEST/HOLD.NoFULL/merge/push/suppressions;groupnosqueeze/files≤400.
ExplicitlyavoidW1auto_lift.rsandprocessorM73/time-estimate.
