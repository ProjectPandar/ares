# MM flush_into_infill object-label RED fixture

`project.3mf` (target, `flush_into_infill=1`, SHA
`c80f9ebb4465e426e6f74b49d1f68c99dc0556b59a24a0f780e9b37e86181c6a`) and
`base.3mf` (control, `flush_into_infill=0`, SHA
`8e09d0d146adf8cd9f444c2c343396e772e508e185a69eefa5700733cb8043d0`) are
byte-for-byte copies of the frozen discovery target/control projects.
Stock machine `Bambu Lab P1S 0.4 nozzle`, process `0.20mm Standard @BBL X1C`,
filaments `Bambu PLA Basic @BBL X1C` + `Bambu PLA Matte @BBL X1C`, two cubes
on extruders 1/2, type2 wipe tower; only `flush_into_infill` differs.

Source: OrcaSlicer pin8500fcdcc `GCode.cpp:5326-5493`.
See `docs/architecture/ard-mm-flush-into-infill-labels.md`.
Both cases use `native_stream_case::check`: missing Native fails, two Native
replays must qualify before complete Native/API/CLI output comparison,
including timing, and raw API/CLI equality. Callbacks only qualify the
witness shape (two-pass marker presence/absence, Native envelope balance);
no Native lines are embedded as output goldens. Expected target failure is
NOT marked should_panic or ignored.

Run `cargo nextest run -p ares-cli --test orca_parity
-E 'test(mm_flush_into_infill_labels)' --test-threads 1 --no-fail-fast`
with the official or seeded coordinator environment, fresh cache/artifact
directories and CPU11.
