# Firmware zero-length layer-change lift: Native-active RED

Source `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
GCode.cpp:5685-5718,7678-7704; GCodeWriter.cpp:1026-1046,626-648.

Input-only ZIP SHA/member manifests are frozen in manifest.json. Geometry is
unchanged from committed marlin2-wipe0.3mf: one overhang model, ByLayer,
Marlin2, one filament, no brim/skirt/support/raft/tower. All cases enable firmware
retraction and retract_when_changing_layer, disable wipe, use Normal Lift and
minimum travel100 to suppress ordinary short-XY travel retraction.

- fw-zero-hop TARGET: configured length0, hop.4.
- fw-positive-hop CONTROL: ONLY length changes to4.
- fw-zero-no-hop CONTROL: ONLY hop changes to0.

Native FIRST must succeed, print all75 model layers with positive extrusion,
emit76 G10 retracts and active G11 unretracts, and emit75 normal lifts for target
and positive-length control,0 for no-hop control. Test then requires validated
public API and actual CLI success, raw API==CLI, and COMPLETE ordered Native
comparison through unchanged golden::compare_ordered_bytes_generator_only.
No M73 filtering or expected-divergence-as-success assertion.

On baseline source, target genuinely FAILS at byte1168/line42: Native M73 P3 R20
versus Ares M73 P3 R19, because later layer-change lifts are missing (Native75,
Ares1). Both controls must whole-output PASS. Production is unchanged in RED.
GREEN only ports the existing boundary lift predicate
according to upstream needs_lift including use_firmware_retraction, with one
predicate source; do not touch estimator/M73, geometry, travel policy or APIs.
No Native outputs are committed.
