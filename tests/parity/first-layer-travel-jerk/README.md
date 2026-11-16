# First-layer travel jerk inputs

Owning source: OrcaSlicer v2.5.0
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`GCode.cpp:10519-10531::travel_to`, `PrintConfig.cpp:4039-4047`,
`Config.cpp:715-738::get_abs_value_at`.
Design: `docs/architecture/first-layer-travel-jerk.md`.

The projects use the public synthetic Ender-3 profile and package structure
from `merged-jerk-tolerance/equal-control.3mf`. Their mesh is replaced by two
disconnected 5 × 5 × 0.6 mm boxes in one object. Local X ranges are [-12,-7]
and [7,12], Y is [-2.5,2.5], and Z is [-0.3,0.3]. The build translation is
(110,110,0.3). Each box has eight vertices and twelve outward-facing triangles.
Layer height 0.2 mm produces three layers.

Common settings: Marlin flavor, default jerk 8, ordinary travel jerk 12,
initial/extrusion-role jerks 8, XY machine jerk limits 20, prime tower off.
Only the initial-layer travel value and the disabled-default control differ.
ZIP entries have a fixed 2026-08-27 timestamp; no generated G-code is included.

| Input | Initial travel jerk | Default jerk | SHA256 |
|---|---|---|---|
| `percent-half.3mf` | `50%` | 8 | `240b0fe31d1cae446d6b79b43ee3705643ef29d27ae7818877773f19417b1654` |
| `absolute-three.3mf` | `3` | 8 | `f4fa248165d43c748933d03d6d43a8773873ce61b47bd0745a050753ac3b6573` |
| `zero-initial.3mf` | `0` | 8 | `2ddd747fedf0c511a1cc3437c3e4594db86f5a25d96f886b48052dfb49ad379d` |
| `regular-control.3mf` | `100%` | 8 | `4abbd244d5d70ed4d1b9a9814189c2e7cd3a8dfb069d8c76f3809c13c199b739` |
| `zero-default.3mf` | `50%` | 0 | `0cd38da5e79a8fe2098a4eeed2a7fe1d92ffa8c1f451ad664a1f4fbd52f470e9` |

Five behavior-named cases in `orca_parity/first_layer_travel_jerk.rs` use the
existing `native_stream_case::check` public Native/API/CLI seam. Each verifies
the input digest, qualifies two complete ordered Native replays, observes
first- and second-layer jerk behavior, then compares complete API and CLI
outputs using the existing generator-only comparator. The caller-selected
external artifact root retains `comparison.json`, `native-replay.json`,
Native output, API/CLI output, and differences for reproducible inspection.
No progress, statistics, motion, or configuration output is masked.

The relative case distinguishes 50% of travel jerk (6) from default jerk (4).
The absolute case selects 3. The zero-initial case makes no travel-jerk request
and keeps the extrusion jerk cached. The 100% control retains ordinary travel
jerk, and the default-zero control disables runtime jerk selection entirely.
Later layers retain ordinary travel jerk where enabled. Nullable project-input
normalization is outside these explicit-value cases, as recorded in the design.

With the project's complete Native environment and external artifact root set,
run `cargo nextest run -p ares-cli --test orca_parity -E 'test(first_layer_travel_jerk)'`.
Execute with both official and seeded launch environments; neither missing
Native producers nor skipped cases qualify as successful parity.
