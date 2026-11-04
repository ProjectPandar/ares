# Resource-part missing build: replacement public slicing witness

Orca8500fcdcc ObjectImporter active1780-1817,
resource dispatch5612-5687/end_model5323 does not require build. Main instance
assembly stays separate. No production changes or Native products committed.

control.3mf is the EXACT fixture
wipe-before-override/control-before50.3mf, SHA
adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2, whose
official+seeded public strict whole-output PASS/rawAPI==CLI is established.
positive.3mf SHA
0eab5d4dea27aa756422e4a821d8d64d1b2ac544afc83dd121ac4899afa33f7a
removes ONLY <build/> in 3D/Objects/overhang.stl_1.model. Manifest pins every ZIP
entry hash: all other content/root build/geometry/settings/IDs/transforms equal.
derive-positive.py reproduces exact positive ZIP bytes from exact control.

This test uses the replacement geometry/settings whole, not selective
M73 disabling. Independent M73 discovery remains deferred; no M73 or comparator
change is in scope. New control must strict-PASS, not merely admit.

Test resource_part_without_build_public_slicing: from_env/reference_enabled,
Native --slice0 --outputdir matching exact qualified source contract (no arrange
flag/loadedJSONs/extraoverrides), validated public API/default overrides/plateNone/
contextseed0/matching generation metadata and real bare CLI.

Every test invocation performs three fresh Native slices and then all three
public API/CLI calls in the same runner. There are no Native-only early returns,
external reference input switches or reference-loading paths. The
reference_enabled/from_env gates are the only optional-oracle gates. References
always come from this invocation's Native commands, never injected/cached output.

Positive and fresh repeat should expose attributable missing-resource-build
admission errors with matching CLI cause vs Native complete G-code. No Ares
stream => rawEq/wholecompare null/NOT EVALUATED, never PASS. Control admission,
API==CLI RAW and strict COMPLETE generator-only Native/API/CLI comparison must
PASS. Every admitted positive after a future correction must also pass.
Motion/arcs/M73/statistics/footer retained; no weakening, skipping/reordering M73,
metadata filters, fixture substitutions or downstream scope growth. ANY residual
G-code difference means STOP/report, no repair within this slice.

Per-leg artifacts: exact sources/digests/Native commands/status/complete snapshots/
streams/neutrality record, public-result.json/API error or stream, CLI output/exit,
full comparator differences and three-leg summary before assertions. No retries.
Fresh cache roots and from_env in both modes.
