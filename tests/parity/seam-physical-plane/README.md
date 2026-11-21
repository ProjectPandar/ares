# Physical Layer::slice_z seam witness

Upstream pin8500fcdcc, `GCode/SeamPlacer.cpp::gather_seam_candidates:1014-1040`
consumes physical `Layer::slice_z` from `PrintObjectSlice.cpp:24-75`.
[SPEC](../../../docs/architecture/seam-physical-plane-resource-spec.md).

Positive is unchanged original d951 resource-build control. Control changes only
modifier part3 sparse density50%→20%, equal to parent normalpart; one ZIP payload
changes, all geometry/global settings/relationships identical. Manifest freezes
whole inputs and every ZIP-entry SHA. No Native-generated products are committed.

Test `modifier_physical_layer_seam_plane_matches_native_public_slicing` runs
fresh Native control→public validated API+real bare CLI→whole strict/raw equality,
then fresh Native positive→both public calls/receipts→target success/raw/whole
assertions. Set `ARES_REQUIRE_ORCA=1` to require Native.
The exact same code must FAIL before the bounded handoff fix and PASS after it.
Missing streams are NOT EVALUATED. No injected reference, Native-only gate,
M73/geometry filter or expected-error PASS. Complete comparison uses the unchanged
project generator-only comparator. `public-attempt.json` records admission;
`public-result.json` records both complete comparisons before assertions.

At41f2, first full strict difference is M73 byte5217/line224; the first motion
difference is third-layer inner hole-wall travel at raw1198, then XY extrusion1203.
The M73 boundary is a total-denominator symptom. The port changes only the seam
caller's physical plane input and deletes its region-height scan helper. Any
remaining whole-file mismatch must STOP, not expand to M73/scoring/perimeters.
Run official and seeded with fresh caches/artifacts/TMPDIR.
