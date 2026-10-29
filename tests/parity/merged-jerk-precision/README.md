# Merged square-corner velocity precision inputs

Source GCodeWriter.cpp315-337 unconfigured ostream; defaultfloat6significant,
Rustdestination merged acceleration.rs square-corner field only. SPEC
`docs/architecture/2026-10-24-merged-jerk-precision-spec.md`.
Base917ec retains prior comment/EPSILON fixes; no production change in RED.

StockEnder3/toweroffKlipper/allrolejerks7/initialtravel100%,default/initialaccel500,
travel700. Sole inner_wall_jerk variant,belowXYlimits8;GUI Tab2568/PrintConfig3317.
No displayed-precision or recommended physical tuning claim.

| Input | Inner jerk | SHA256 |
|---|---|---|
| equal-control.3mf | 7 | 173b2cd1e4d549de65953512f9d8d3d853f3753f68ff0546b0be63368987a78b |
| precision-target.3mf | 7.0002 | 8b1a2d4927db6ce127d8f8392fab228a04485922a0db741de43d5a93d79fdee3 |
| exact-milli-control.3mf | 7.002 | 53ddea7da1ffd0262c4a04d9c83913adc0b46167349950732b4613099a584991 |

Three direct existing native_stream_case calls,no new helper. Qualify twofresh
Native complete outputs; callback reads real saved innerjerk from input ZIP JSON
and checks it occurs among actual Native square-corner words,without Native
numerical/coordinate/count constants. Whole API/CLI output comparison unchanged,
generator-lineonly normalization,nofilter/mask. NoNativeoutputscommitted.
Expected RED target7.0002 becomes7 in Ares; two controls fullPASS bothmodes.
Failureclasses beforecode in SPEC:fixed3vs6significant,format-before-admission,
changing command counts/cache,globalformatter or adjacentfields changes.
GREEN only the one field's existing formatter call; freezeinputs/tests.
ACCEL_TO_DECEL remains deferred. NoFULL;groupregistrationno squeeze.
