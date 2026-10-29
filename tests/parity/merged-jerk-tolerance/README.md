# Merged Klipper jerk tolerance inputs

Source GCodeWriter.cpp333-338, libslic3r.h52/306-309; source-cited SPEC
`docs/architecture/2026-10-24-merged-jerk-tolerance-spec.md`.
Base frozen comment062c; no production changes in this RED.

StockEnder3/toweroff Klipper, default/initialaccel500/travel700, allrolejerks7,
initial_layer_travel_jerk100%. Only inner_wall_jerk varies, below machine limits8.
GUI Tab2566-2568/PrintConfig3317-3324. Numerical saved-option edge case, not claim
about GUI display precision or recommended physical tuning.

| Input | Inner jerk | SHA256 |
|---|---|---|
| equal-control.3mf | 7 | 173b2cd1e4d549de65953512f9d8d3d853f3753f68ff0546b0be63368987a78b |
| within-native-tolerance.3mf | 7.00005 | e407c7c7ea2f19cfd1436c7406b1d8f31b1fd72f1a776c60e814bfb20a276f60 |
| within-both-tolerances.3mf | 7.0000001 | df9e6effe76a115996017892f4b488764add7073744619f7d69a4b7adabd9d4b |
| above-native-tolerance-clean.3mf | 7.002 | 53ddea7da1ffd0262c4a04d9c83913adc0b46167349950732b4613099a584991 |

Four direct existing native_stream_case calls, no new helper. Qualify fresh
Native double replay then parse observed square-corner velocity words: actual
values must occur; only above control varies between successive Native values.
No hardcoded Native numeric/count/coordinate goldens. Then complete unmasked
API/CLI comparison via unchanged generator-only comparator, all output included.
Target RED expected extra duplicate commands; three controls fullPASS bothmodes.
Callback observations do not filter comparison or prove all private cache states.

Failure modes documented before implementation in SPEC: tolerance/admission,
cache update on suppression, compare-before-clamp mistake, rounded/relative
comparison, suppressing true changes. No exact decimal7.0001 boundary assertion.
7.0002 is NOT included: it exposes independently deferred writer precision mismatch,
recorded externally; the clean above control7.002 avoids that confound.
No Native outputs committed; grouped registration no root growth/squeezing.
