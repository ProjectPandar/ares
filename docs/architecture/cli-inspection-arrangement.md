# STL placement before CLI inspection

Source: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

## Upstream behavior

- `src/libslic3r/PrintConfig.cpp:12857–12878,12933–12939` registers `--info`, `--inspect-mesh`, and `--inspect-paint` as actions and `--arrange` as a transform. `src/OrcaSlicer.cpp:1404–1448` rejects other actions such as `--info` alongside either inspect action before loading. It checks `m_actions`, not `m_transforms`, so these three reports can each be requested with `--arrange`.
- `OrcaSlicer.cpp:1749–1758` loads with default instances; `src/libslic3r/Model.cpp:805–811,1554–1563` adds an instance only to an object that has none. Fresh STL input enables arrangement (`OrcaSlicer.cpp:1924–1926`) and has a separate vertical drop at `1969–1978` (`Model.cpp:1896–1921`).
- `OrcaSlicer.cpp:4909` runs transforms before arrangement and the report actions. At `5051–5066`, `--arrange 0` clears `need_arrange`, `--arrange 1` sets it, and other values leave it unchanged. For a standalone STL without other placement requests, `0` bypasses the automatic arrangement branch at `5619–6270`. This skips the arrangement's instance placement, not explicit `--center` or `--align-xy` (separate transform branches at `5027–5049`) or the independent `ensure_on_bed` pass at `6272–6281`.
- The action loop starts at `OrcaSlicer.cpp:6297`: `--info` prints at `6335–6340`, `--inspect-mesh` prints and returns at `6341–6363`, and `--inspect-paint` prints and returns at `6364–6387`. The order is transform, placement, then reporting regardless of where `--arrange` appears relative to the action in argv.
- `src/slic3r/Utils/MeshInspect.cpp:34–48` reports both object bounds and first-instance bounds and offset. In contrast, `Model.cpp:2564–2581` prints bounds of `raw_mesh()` (`1690–1700`, volume transforms only), and `src/slic3r/Utils/PaintCLI.cpp:142–170` measures mesh-local vertices, not instances. An instance translation is visible in mesh inspection but not in either of the other reports.

## Rust boundary

`crates/ares-cli/src/main.rs` calls `SliceArgs::parse_transforms` before dispatching all three reports to `inspect::run`. That handler passes the parsed transforms to `ares_core::inspection::load`, whose standalone STL branch builds a `ProjectInstance` with an unconditional bed-center translation before `CliProjectTransforms::apply`. The transform dispatcher already runs explicit arrangement only for `Some(1)`, so it cannot undo that initial translation when the option is `0`.

Apply the upstream default-instance placement rule in `crates/ares-core/src/project/domain/inspection.rs`: disable the automatic XY placement for a standalone STL when the parsed arrange value is `0`, retaining the independent Z drop. Preserve volume transforms, default/forced placement, existing 3MF instance placement and arrangement, and existing report formatting. The same instance is prepared for all three report actions; no report-specific geometry compensation is needed. The existing STL centering path for automatic arrangement remains a compatibility shell, not a new arrangement implementation. General layout/duplication, explicit centering and alignment, and normal slicing entry points are out of scope.

## Observable contract

Use a synthetic STL to compare each accepted report (`--inspect-mesh`, `--info`, `--inspect-paint` individually) with `--arrange 0` and with `--arrange 1`. Compare stdout, stderr, exit status, and the list of produced files against a live Native run, and verify the Native replay. In mesh inspection, disabling arrangement preserves the input XY bounds while enabling it places the instance; the object bounds are unchanged. The other two reports should match Native for both values without any formatter changes.
