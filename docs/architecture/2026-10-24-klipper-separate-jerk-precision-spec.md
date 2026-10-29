# SPEC: separate Klipper set_jerk_xy ostream precision

Base frozen d0e8a51000f9ac66b954a0ee4b179f380d949452. Coordinator ACK:
SPEC → Native-first public RED → one-field GREEN → review/hold. No FULL.

## Upstream rewrite boundary

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:262-270::set_jerk_xy` creates ordinary
ostringstream, clamps XY and streams double jerk as standalone
SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY. Defaultfloat six significant digits.
Rust destination ONLY `gcode_emit/motion/jerk.rs` Klipper arm22-27 formatter,
currently axis(x). Reuse existing format::ostream; no new helper/formatter/API.
Existing Ares writer shell follows this source, not independent axis quantization.
Included: ONLY this field axis→ostream. Deferred: cache/tolerance/clamps/comments,
merged writer, Repetier/other flavor arms, formatter implementation, processor/
templates/fan/otherowners. Preserve all approved frozen slices unchanged.

## Witness / GUI provenance

StockEnder3/toweroffKlipper/commondefaultjerk7, ONLYinitial_layer_jerk varies
7/7.12345/7.125. Stockdefaultaccel0 yields real separate first-layer jerk command
without ACCEL word. Target belowXYlimits8, so no clampconfound. Realinitial-layer
jerk processsetting (PrintConfig after3346, GUI Speed/JerkXY). Savedfloating
edgecase, no claim about displayeddecimals or physicaltuning recommendation.
Target firstnormalizeddifferencebyte1237:Native standalone word7.12345,Ares7.123,
onlyonecommandline. Two controls fullPASS;6freshNative slicesstable/allCLI0.
Exactd0e8binaryused;discoveryevidence
`/home/indexyz/ares-evidence/w4-klipper-separate-jerk-precision/PROPOSAL.md`.
No API/seeded/RED/FULLclaim fromdiscovery.

SHA:
- integer7:3a8b3f3fbbcbdc9b7b5eb1ee13e5e08533ae8e54d60fd814be46ff45b1c02114
- target7.12345:68ec6624b778a08282b7767a1cac831bd27db075c08cd59249dc8734b7058f12
- milli7.125:d07cef12a7b998feb5801acb41eb3c68da13cbb5517916cb105270b8f7f2636a

## RED-first failures and exits

ThreeSHAinputs/directexistingnative_stream_casecalls,nohelper. FreshNative
doublereplay thencallback reads savedinitial-layerjerk and observes actual
standalone square-corner word retainingvalue, explicitly excluding ACCEL-bearing
merged lines. NoNativecoordinate/count/valuegoldens,outputsnotcommitted.
Then wholeunmaskedAPI/CLIcomparison/generator-lineonly,allcommands/timing/footer.
Failureclassesbeforecode:axis3vsostream6,wrongmergedpath,formatbeforeclamp/cache,
otherflavor/helperchange,losingcontrols. CommitREDbeforeproduction.
CPU17PIN17jobs1threads1officialthenseeded:onlytargetFAIL/controlsPASS,
Nativequalification+API==CLI. GREENsingleformattercall,tests/inputsfrozen;
fmt/workspacealltargetclippy/threecasesbothPASS,REVIEW-REQUEST/HOLD.NoFULL/
merge/push/lintsuppression,groupregistrationno squeeze,touchedRust≤400LOC.
