# ACCEL_TO_DECEL precision parity inputs

Source GCodeWriter.cpp241/324, unconfigured ostream in both acceleration writers.
SPEC `docs/architecture/2026-10-24-accel-to-decel-precision-spec.md`, frozenbase6bc.
No production changes in this RED. StockEnder3/toweroffKlipper/default&initial
accel500/travel700/enableacceltodecel1; ONLY real factor option varies.
PrintConfig3272-3287 coPercent1..100;GUI Tab2559-2560. No UIdecimaldisplayclaim.

| Input | Factor | SHA256 |
|---|---|---|
| factor-control.3mf | 50 | a1c7580b4a1e596fa8fdd72ba2301d95b6b6539a2cd74e9ee612f6d930c6091d |
| fractional-factor.3mf | 12.34567 | 56cb137c5fdf3db5899ce19146a10bbd2b263f6527e7ebff2fd70881fdbf0927 |
| exact-decimal-control.3mf | 12.34 | c73b59deb59296e87c8863ad3048e401876079791c2f282eb273ca1134f577ec |

Three direct existing native_stream_case calls/no new helper. Native replay then
callback parses actual ACCEL_TO_DECEL/ACCEL words and real input factor; verify
six-significant output via test-local scientific formatting of same double
expression, requiring multiple observed acceleration transitions. No Aresproduction
formatter called by callback/no hardcoded Native coordinate/count/value goldens.
Then complete unmasked API/CLI comparison using existing generator-only seam,
all commands/timing/footer/config included;noNativeoutputscommitted.

TargetRED expected fixed3truncation in standalone first and merged later lines,
two controlsfullPASS. Failureclasses beforecode in SPEC:merged-onlyfix,arithmetic
reordering,cache/clamp/gates/neighboringwordchange,decimaltieassumption. Observed
Native61.7283mustnotbehardcoded. GREEN only2formattercalls;freezeinputs/tests.
NoFULL,groupregistrationwithoutrootgrowth/squeezing.
