# P1S ByLayer brim RED fixture

`project.3mf` is a byte-for-byte copy of
`/home/indexyz/ares-evidence/worker1/sequential-next-discovery-117f4ffe3/p1s-bylayer-feature-control/project.3mf`.
SHA256: `dcae275709dc16016c60308ae42d8523385dbabc18e5d31448f0baceb6e2d36a`.
No project settings or mesh edits were made.

Source: OrcaSlicer pin8500fcdcc Brim.cpp:813–856.
See `docs/architecture/ard-p1s-bylayer-brim-contour.md`.
Both cases use `native_stream_case::check`: missing Native fails, two Native
replays must qualify before complete Native/API/CLI output comparison,
including timing, and raw API/CLI equality. No Native motion lines are
embedded as goldens or asserted by callbacks; the full comparator is the gate.
The passing control reuses `validation/w5-brim/outer-only-w5.3mf` without changes;
it is an OuterOnly materialization control, not a matched P1S settings control.

Run `cargo nextest run -p ares-cli --test orca_parity
-E 'test(p1s_bylayer_brim_)' --test-threads 1 --no-fail-fast` with the official
or seeded coordinator environment, fresh cache/artifact directories and CPU11.
Expected target failure is NOT marked should_panic or ignored.
