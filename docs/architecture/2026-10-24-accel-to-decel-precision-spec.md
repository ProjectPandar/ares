# SPEC: Klipper ACCEL_TO_DECEL ostream serialization

Against6bc322919827b502fcd389927b272b3e1ea20174:
SPEC → public Native-first RED → two-call GREEN.

## Source-owned boundary

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:216-249::set_acceleration` at241, and
`:311-338::set_accel_and_jerk` at324, both stream the double expression
`acceleration * config.accel_to_decel_factor / 100` through an unconfigured
ostringstream (defaultfloat six significant digits). No fixed axis quantization.
Rust destination `gcode_emit/motion/acceleration.rs:55,109`, ACCEL_TO_DECEL value
format calls only. Existing `motion/format.rs::ostream` already owns the ported
serialization; use it, not a new formatter, helper or Ares-specific policy.

Included: switch both calls from axis to ostream. Deferred: multiplication/
division order, arithmetic precision, clamp/cache/admission, enable gates,
comments, neighboring fields, formatter implementation and all other owners.
The first standalone layer-start difference and later merged bridge difference
must both be covered. Prior slices remain unchanged.

## Witness / GUI provenance

StockEnder3/toweroffKlipper, default&initialaccel500/travel700, enable1. Sole-key
accel_to_decel_factor variants50 control,12.34567 target,12.34 exactdecimal.
`PrintConfig.cpp:3272-3287` bool/defaulttrue and coPercent/min1/max100/default50;
GUI `Tab.cpp:2559-2560`. Target is allowed saved fractional percent; no assertion
about displayed decimals or recommended physical tuning. Geometry/templates/
jerk/hop/PA/temperature/comments otherwise stock.

Target first normalized differencebyte1240: Native ACCEL500 ACCEL_TO_DECEL61.7283
versus Ares61.728; later merged ACCEL250 Native30.8642 versus Ares30.864.
Two controls fullPASS, six fresh Native slices replaystable; exact6bc existing
actual CLI. Native61.7283 reflects binary double arithmetic then ostream, not an
exact decimal rounding policy.
No API/seeded/RED qualification inferred from discovery.

SHA:
- factor50:a1c7580b4a1e596fa8fdd72ba2301d95b6b6539a2cd74e9ee612f6d930c6091d
- factor12.34567:56cb137c5fdf3db5899ce19146a10bbd2b263f6527e7ebff2fd70881fdbf0927
- factor12.34:c73b59deb59296e87c8863ad3048e401876079791c2f282eb273ca1134f577ec

## RED-first plan / failure modes / exits

Freeze three SHA inputs and direct existing native_stream_case calls. Native
double replay first; callback reads factor from saved input and checks actual
ACCEL_TO_DECEL word against six-significant formatting of actual ACCEL*factor/100,
with test-local standard scientific formatting only (not Ares production writer).
Require both ordinary acceleration and merged bridge setting via multiple
observed distinct accelerations, without hardcoded Native counts/coordinates.
Then complete unmasked API/CLI comparison, all timing/config/footer included.
No new test helper, Native outputs/goldens or comparator changes.

Pin before implementation: fixed3 instead of6significant, fixing only merged
consumer, changing arithmetic order or caches, decimal tie assumptions, changing
neighboring fields/comments/enable handling. Source review must verify same
expression before/after; do not hardcode observed61.7283.

Official then seeded. RED only targetfails/twocontrols
PASS, Native replay/callback/API==CLI qualify. Commit RED before production.
GREEN only two format calls; tests/inputs frozen. fmt/workspaceall-targetclippy,
three cases bothmodes PASS. No helper/
lint suppression, group registry/no squeezing, all touched Rust≤400LOC.
