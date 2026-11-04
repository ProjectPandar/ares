# J1 runtime usage mask: frozen public-input E2E cases

Source/spec: `GCode.cpp:2901–2904`, upstream8500fcdccaa10b5099ac20d252af3a7c560046f1;
`docs/architecture/gcode-runtime-used-extruder-mask-j1.md`.

`role2.3mf` is the exact original finding input, SHA256
7d796432a448d46adc30e9ea91b4c729a6044058d3dd61bb4f5c13e64ea615f4.
No fixture tuning: stock J1 startup, unchanged Dumbbell geometry, two0.4
nozzles, explicit role IDs2, object default extruder1, Manual map1/2.
Independent fresh seed0 public-API/Native complete-byte RED confirms
Native usage false/true versus Ares true/true, initial1 in both. The true
first difference is byte461/line18, M73 P0 R2 versus P0 R3. M73/footer remain
in the assertion, generated-by identity is the only normalization.

Three controls were frozen BEFORE their first Native comparisons, changing
only `Metadata/project_settings.config`:

| Case | SHA256 | Declared change | Expected usage/initial |
|---|---|---|---|
| role1 | e74bf5ed81edcf4c00ea36be06967e1447ee4d61d25cd20b23fd39f16f6227e5 | all six role selectors1 | true,false /0 |
| both-used | 12cf6020f118d5a2c5b5aa081f5dd9a4a3c043f0c3788558c13fa80687a4f3bd | inner-wall selector1, outer remains2 | true,true /1 |
| by-object | 828e4056bfdeaaed2f028fdbd596355a2f2fa56d0c8fab060f89268435823633 | role1 selectors plus by-object print sequence | true,false /0 |

The first raw full-byte run recorded all four DIVERGENT honestly. Role1
and by-object differ at byte2449/line165, Native M73 P41 versus Ares
WIPE_START. Both-used differs at byte2384/line189, Native
FORCE_RESUME_FAN_SPEED versus Ares blank/change T0->T1. Their usage, initial
and tool/flush witnesses are already equal, so these residuals are not mask
REDs. Only these three controls assert exact usage/initial/literal
tool+flush witnesses WITHOUT full-byte comparison.
The primary still requires complete-byte RED→GREEN; any subsequent residual
must be reported without stripping it. Inputs and expectations were not tuned.

The independent residuals (wipe progress, filament-start transition) are
recorded externally with exact inputs/outputs/first differences and hashes.

The final test-only scope run has one primary complete-byte RED and three
witness-only PASS. Do NOT interpret those PASS as complete-stream parity.
Four fresh Slice records bind input/reference hashes and seed0, with zero
timeouts; compilation was outside each shared300s case deadline. No
production code changed.

Tests: `crates/ares-cli/tests/orca_parity/runner/j1_used_extruder_mask_follows_emitted_tools.rs`.
Use the configured pinned Native/resources/artifact environment and:
`cargo nextest run -p ares-cli --test orca_parity -E 'test(j1_used_extruder_mask_follows_emitted_tools)'`.
Artifacts retain complete streams for every case, witness JSON, Native
command/cache receipts and primary full-comparison manifest. The record
`docs/architecture/gcode-runtime-used-extruder-mask-j1-red-record.json`
binds this independent scoped RED. No private emitter/test-only production
API, ignored test, panic patch or Native rebuild.

## J1c final Top-surface lifecycle supplement

`top1-control.3mf` SHA256
83d0c1908087d796008602671bdc3a6ef43f12fdd7423b0d3c96db2d423af310
changes only `top_shell_layers` from `0` to `1` in the original role2
configuration. All other configuration values and ZIP entry payloads are
unchanged. This separate control was frozen before its first Native call;
the original primary input remains unchanged.

`runner/avoid_crossing_routes_around_top_surfaces.rs` calls the public `slice_project` API
and compares the last layer's XY travel commands against fresh Native.
The zero-top case asserts the frozen Native shortcut through
`X152.911 Y103.089`, demonstrating removal of the obsolete top-surface holes.
The nonzero-top control preserves avoidance of surviving Top regions.
These are **travel-only lifecycle witnesses**, not relaxed complete-byte
comparisons; the original primary still asserts the entire file.

Fresh seed0 runs gave:
primary complete-byte RED at byte3810/line222; zero-top travel RED; nonzero-top
travel PASS, with zero timeouts. No J1c production edit. Tracked machine record:
`docs/architecture/gcode-runtime-used-extruder-mask-j1c-red-record.json`.
