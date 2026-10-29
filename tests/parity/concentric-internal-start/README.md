# Concentric internal-solid loop-start public RED

SPEC `docs/architecture/2026-10-25-concentric-internal-loop-start-spec.md`
(base `5e8be001f`). Owning upstream: `Fill/FillConcentric.cpp:105-145`
standard `ipConcentric` Arachne variant loop-start rotation gate
(`extrusion->is_closed` only), `Arachne/WallToolPaths.cpp:573-670`
`stitchToolPaths` closed-chain classification, `Arachne/utils/ExtrusionLine.hpp`
`to_thick_polyline`. Rust destination:
`fill_entities/concentric/finalize.rs::finalize_standard_polylines` gate fed by
`arachne/wall_toolpaths/stitch.rs` chain closure; GREEN edits only the proven
`stitch.rs` classification after the C++ `is_closed` capture.

Inputs derive from the official-exported stock fixture
`tests/parity/fan-mixed-timing/control.3mf`
(SHA256 `b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298`)
by mutating exactly one `"key": "value"` token in
`Metadata/project_settings.config`; every other ZIP member is byte-identical.

| Input | `internal_solid_infill_pattern` | SHA256 |
|---|---|---|
| `control.3mf` | `monotonic` | `b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298` |
| `target.3mf` | `concentric` | `2407dd48fea1529b51947958c6ea62d577d61f6b8173a989c1b4113983b6c6f6` |

Expected at RED: control PASSes whole ordered output against runtime Native;
target FAILs — only the outermost loop of the internal-solid concentric
islands (Z1/Z6) starts at a different junction (Native: raw walk start; Ares:
chained-nearest rotation of a ring upstream leaves open). Native artifacts are
generated at runtime and never committed; without a Native oracle the case
errors out rather than passing silently.
