# Deferred-lift EPSILON public RED inputs

Upstream `libslic3r.h:52`, `GCodeWriter.cpp:625-648`, `685-808`.
SPEC `docs/architecture/2026-10-24-deferred-lift-epsilon-spec.md`.

Stock `tests/parity/orca_cli_ender3.3mf`, prime tower disabled identically;
change only printer `z_hop`. Geometry, all templates and other settings unchanged.
Schema: `PrintConfig.cpp:5255-5264` floating height, min0/max5, no positive floor.
GUI option: `Tab.cpp:5152`. Submicron targets intentionally test a valid numeric
boundary, not recommended printer settings; UI display precision is not asserted.

| Input | Height mm | SHA256 |
|---|---|---|
| stock-control.3mf | 0.4 | a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae |
| zero-control.3mf | 0 | 6ec170cd2be5d9598f978c3363587e571814ab118b704a81b636c53c71244f8d |
| below-epsilon.3mf | 0.00005 | 7614d97c841ae5300be6f4aa219e05241ef6f876c199a084a38afc323fc682c5 |
| at-epsilon.3mf | 0.0001 | efec866bdcb60c0e523c064fa73bb298eb11298c469dad7610e1e6d01e1892ba |
| above-epsilon.3mf | 0.00011 | a5320b5b36a1596031dc0d6bb25c74447ccfbb948169ebfdf1f0b6192aa9c2fa |

`native_stream_case::check` first creates and qualifies two fresh Native slices,
then the callback checks that the first approach after the layer marker is XY
for zero/below/equal versus Z-only for above/default. These are structural
observations of Native output, not checked-in Native streams or coordinates.
Then unchanged strict generator-only comparison checks complete public API and
CLI streams, including M73/timing. No filtering or mask. Artifacts retain full
streams and replay/comparison JSON under the selected external artifact root.

Expected frozen RED: below/equal diverge; three controls pass in both modes.
Native API/CLI equality and replay must qualify independently of that expected
failure. First whole-output target difference may be an earlier M73 timing
consequence of later extra Z moves; it must not be concealed.

Failure classes before implementation: strict > versus >=; early clearing of
subthreshold pending state; lazy_lift incorrectly rescheduling while pending;
first-position consumption bypass; ordinary travel classifying scheduled as
consumable; zero and above/default regressions. Public bytes cannot alone prove
the private pending state persists: future port review must verify that source
invariant explicitly. No GREEN change accompanies these cases.

Cases live in existing `object_emission_cases.rs` grouping (object/retract cases)
with an ordinary separated registration block; neither root registry changes.
