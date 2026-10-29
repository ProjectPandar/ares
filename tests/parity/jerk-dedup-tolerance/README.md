# Writer jerk approximate-dedup public RED inputs

Source `GCodeWriter.cpp:255-260`, `libslic3r.h:52,306-309`.
Approved SPEC `docs/architecture/2026-10-24-jerk-dedup-tolerance-spec.md`.
The test stack sits on approved frozen lift tip `2692155a9`; its production tree
is unchanged by these RED tests. Temperature `a5cc569c0` remains a separate frozen
chain until the coordinator's combined integration window.

All inputs derive from stock `orca_cli_ender3.3mf` with prime tower disabled.
The equal-control sets these real process settings to7:
`default_jerk`, `outer_wall_jerk`, `inner_wall_jerk`, `top_surface_jerk`,
`infill_jerk`, `initial_layer_jerk`, `travel_jerk`; initial-layer travel jerk100%.
All other controlled variants differ ONLY in `inner_wall_jerk`. Geometry,
templates, temperatures and stock0.4 hop unchanged. Both machine XY limits8
remain above requested7.x values, so the witness does not depend on clamping.

| Input | Inner jerk | SHA256 |
|---|---|---|
| stock-control.3mf | stock profile | a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae |
| equal-control.3mf | 7 | afe37155ecbdb81ebe72e49a8e5dd5709b359f0e63f73d0e50a3689f12d2fe2b |
| within-native-tolerance.3mf | 7.00005 | e7a5ca34d4981a559d82d6895f59f5a642ddbdb69e49aa715aef5bd5eb351a80 |
| above-native-tolerance.3mf | 7.0002 | 64abf2e38a99dfe87d84d3380149794bfd37779bf6d3d8821445e22bb0c66847 |
| within-both-tolerances.3mf | 7.0000001 | 335b65c5f33a7ca0a71f72b926c325965179ef3e6eca912a7bbb341f382f33ff |

Native accepts these floating values (`PrintConfig.cpp:3317-3324` coFloat/min0),
GUI option `Tab.cpp:2568`. This is a numerical-boundary regression, not a claim
about displayed GUI decimal precision or a recommendation for physical tuning.

`native_stream_case::check` qualifies two fresh Native complete streams before
callback assertions: body M205 commands must exist; controlled near-equal values
must not change body jerk commands, whereas the above-tolerance control must.
These observations compare Native commands to each other, without hardcoded
Native command text, coordinates or counts. Stock only requires the machine jerk
configuration path to be exercised; it emits no body M205 in the stock Native
reference. Callback inspection is NOT a comparison filter: complete
public API+CLI bytes, including all timing/config/footer, then pass through the
unchanged generator-only comparator. No committed Native outputs/goldens.

Failure classes pinned before implementation:1e-6 instead of upstream1e-4;
unconditional suppression of admitted changes; cache updating on skipped calls;
comparing rounded or clamped output instead of requested values; treating decimal
7.0001 as an exact binary subtraction boundary. Cache-update ordering still needs
source review; these public cases alone do not prove internal cache state.

Expected RED: one within-native target fails; four controls pass both modes,
Native replay is stable and raw API==CLI throughout. Register under the existing
object/writer-motion group with a separated block, never squeeze root registry
lines. No GREEN change or new EPSILON constant accompanies this test commit.
