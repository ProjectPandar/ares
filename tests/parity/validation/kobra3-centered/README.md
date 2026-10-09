# Kobra 3 centered input contract

These source inputs exercise the configured bed polygon and final brim printable-region
subtraction. Native reference streams are generated at test runtime; this directory
does not supply captured output receipts or historical comparison expectations.

## Input provenance

The four centered variants change only the build-item translation in
`3D/3dmodel.model`: `(5,-56,5)` becomes `(127.5,127.5,5)`. Grounded Z,
mesh/component transforms, profiles, templates, IDs, and the other eight ZIP
members remain unchanged. Local mesh bounds are `(-5,-5,-5)..(5,5,5)`.
Original world bounds are `(0,-61,0)..(10,-51,10)`; centered world bounds are
`(122.5,122.5,0)..(132.5,132.5,10)`. Bed XY bounds are `(0,0)..(255,255)`
and plate origin is `(0,0,0)`. The inputs use Klipper and relative E.

The hashes below identify source inputs, not Native output.

| Centered input | Original input SHA-256 | Centered input SHA-256 |
| --- | --- | --- |
| `0.2.3mf` | `707ec7efeab7743a52cabc851a024c9408f32fd90e98d5abebceb1812ad341e7` | `d63ac67f57d8cfcc36d4c8cbf62fb11ad436f7893454c8bba2760f52ec61c51c` |
| `0.4.3mf` | `831fba613b6ba23373fe884aa5abc5601f4d68d46da5f50c545e88c64a43f742` | `f1e8dfd94990ed53e5180e699e28cd460da9c315d7c8a2464af094f8481254cc` |
| `0.6.3mf` | `9e763ea5197aaa396ee85236031695394422a7a251ef7e7f4b794eeac6f276cd` | `9bcf3a21d416f94a707eba82149fba6eb58cd815918a6369c98d5e345e3a3c06` |
| `0.8.3mf` | `cfc7f9345ae29eabb4363ad118577d2893a0d0db2bc4bd71f404c0ae89918a14` | `0dc37a07daedf1aa01a48832a2ff3b4e70f6e367075274391e42884a98db1b0c` |

`0.2.3mf`, `0.6.3mf`, and `0.8.3mf` select `brim_type=outer_only`.
`0.4.3mf` retains `auto_brim`. Its explicit outer-only derivative changes only
`Metadata/project_settings.config` key `brim_type` from `auto_brim` to
`outer_only`; the other eight ZIP members remain unchanged. The handwritten
recipe is retained in `0.4-outer-only-provenance.json`. The derivative
`0.4-outer-only.3mf` SHA-256 is
`7a6e8d8fb0eef2a5467f23ea979b9e9822c42b97b8cc97743ac29cb29080c49c`.

## Source ownership and runtime verification

Upstream revision: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `libslic3r/Model.cpp:2976,3014-3026`, `Model::setPrintSpeedTable`: configured
  bed polygon with excluded regions subtracted.
- `libslic3r/Brim.cpp:421,643-683`, `outer_inner_brim_area`: per-extruder
  printable bed regions and final object/support brim subtraction.
- `slic3r/GUI/PartPlate.cpp:2510-2520`, `PartPlate::set_print`, and
  `libslic3r/Print.cpp:2618,2636-2638`, `Print::export_gcode`: plate origin
  forwarded to G-code export offset.

`crates/ares-cli/tests/orca_parity/brim_bed_region.rs` owns four runtime checks:

- `brim_bed_region_kobra3_02_outer_matches_native`
- `brim_bed_region_kobra3_04_outer_matches_native`
- `brim_bed_region_kobra3_06_outer_matches_native`
- `brim_bed_region_kobra3_08_outer_matches_native`

Each checks its input SHA, generates a fresh Native stream using the existing
`OrcaRunner`/`stages::run` adapter, asserts Native brim activation, and compares
the entire stream using the existing generator-only normalization. The 0.4
outer test uses `0.4-outer-only.3mf`. Missing Native follows the shared policy:
required mode fails; optional mode reports a visible skip. Runtime artifacts
remain outside the repository. Brim controls, processor tests, and WASM input
consumers retain these same source inputs.
