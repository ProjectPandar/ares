# Type2 effective retraction inputs

## Upstream boundary

Port the input boundary of OrcaSlicer2.5.0 WipeTower2, not a new retraction or
preheat algorithm:

- `PrintApply.cpp:295–317::print_config_diffs` selects filament-prefixed
  overrides for extruder retract keys and their filament-map indices.
- `PrintConfig.cpp:12356–12387::compute_filament_override_value` clones the
  machine option and applies the nullable filament override.
- `PrintApply.cpp:1665–1671,1690–1692` applies these effective values to
  `Print::m_config`, independently of the full configuration retained for export.
- `Print.cpp:5515–5523` passes that effective config to Type2 `set_extruder`.
- `WipeTower2.cpp:1225–1226::set_extruder` stores effective retraction length and
  speed; it does not read filament-prefixed keys itself.

Rust already ports the override merge in `options/project_config_views/retract.rs`
into `views.runtime`. Type2 planning currently supplies runtime speed but reads
full-config length inside `wipe_tower/filament.rs::Filament::from_config`.
Replace that mixed input with one borrowed runtime-config source at planning,
passing its per-tool length/speed pair to the constructor. Keep all other
filament properties on their existing full-config input. Do not duplicate
nullable/map resolution in the tower or mutate the full configuration.

## Included consumers and deferred behavior

The shared Type2 Filament feeds `wipe.rs::scrub_entry`
(`WipeTower2.cpp:1935–1955`) and `finish_layer.rs` wall path generation
(`WipeTower2.cpp:2652–2683`, writer retract/unretract926–928). Both must consume
the same effective pair. An audit must retain all references to these fields.
Type1 owns a separate filament constructor and is deferred; do not change it
without its own upstream input admission. No generic motion retraction,
ExportLines, cooling, timing precision, fixture or comparator change is included.

## Failure witness and verification

Retain the existing public two TiQ2 whole-output tests and unchanged fixture.
Before changing production code, confirm that the tests fail with the T0 preheat
as the first mismatch: Native at line 5246 and time 30 s, Rust at line 5236 and
time 31 s. A read-only trace verified against the complete streams shows four
tower E-only moves of 4 in Native versus 0.8 in Rust between T1 and T0. T1
timestamps match, and Rust T0 is 0.511993408203125 s shorter. The
threshold crosses the same1.009s travel, accounting for the ten-line insertion
jump. This is an input difference, not a reason to change the backtrace consumer.

After the input fix, require the original first mismatch to advance beyond5236
without relaxing full-stream/line-count/config/footer/marker assertions or the
existing generated-by-only normalization. Repeat the same110 test identities in
both runtime modes with zero P→F transitions, report every remaining failure,
and retain repeatable artifacts. Verify formatting, workspace all-targets
Clippy, core wasm32 and diff checks. If the original E2E tests still fail,
full-stream parity has not been established. Existing Ares constructors remain
a compatibility shell around this exact upstream configuration admission, not
a separately designed pipeline.
