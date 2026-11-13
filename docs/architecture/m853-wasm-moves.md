# M853 second Rust slice: convert and thin WASM move exposure

Source owner: OrcaSlicer8500fcdcc,
`src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp::convert`191–268,
role/type conversions70–145, color conversion55–67; destination
`ares-vgcode::convert`. This is the rendering-neutral mapping into upstream
libvgcode GCodeInputData/PathVertex, not a new slicing or preview pipeline.

## Decisions

- Keep existing byte-returning WASM slicing entry points unchanged. Add
  sliceProjectPlateProcessed returning SliceOutput. Web integration and
  deletion of temporary TS conversion are a separate web slice.
- Core's existing processed slice API is the only slicing/recording owner.
- Convert skips initial dummy move and inserts upstream phantom path starts
  at previous position with zero time when type/role/flow changes.
- Accept explicit tool/color-print palette strings as upstream convert does.
  Caller supplies project/preset filament_colour (the GUI owner), not empty
  processor extruder_colors. Empty color-print palette uses tool palette.
- WASM exposes raw move buffers for lossless consumer/oracle verification,
  plus converted vertex buffers. IDs remain u32, role/type/tool/color u8,
  numeric fields f32; no JSON numeric roundtrip or precision loss.
- Buffer packing is adapter-only. No duplicate G-code parser in WASM or TS.
  Take methods transfer owned Vec data; callers can avoid copying raw moves
  when only converted vertices are needed. Each buffer's layout is documented.
- Deferred fields from first slice remain deferred: actual_feedrate values,
  stealth times, layer_duration, PA/accel/jerk, object labels/print_z.
  No rendering implementation or M854 override API in this slice.

## Test-first plan

1. Add tests for no moves/dummy, move/role/flow path boundaries, phantom
   coordinates and times, scalar metadata, palettes and color parsing.
   Capture missing convert API RED, then port the retained handoff WIP and
   reconcile against the exact upstream code (including invalid colors).
2. Add thin SliceOutput/move/vertex buffer bindings with packing tests and
   public processed entry. Expose settings through existing core API only.
3. Browser E2E calls actual compiled WASM with Ender3 and KSR projects.
   Use returned G-code with the existing --process-gcode --dump-moves oracle.
   Compare all19 raw fields using exact integers/copied f32,
   1ULP derived fields and2ULP time; preserve full CSV/diff artifacts.
   Verify phantom mapping independently against oracle records as well.
   Missing oracle is not passing browser evidence.
4. Run workspace Nextest, Clippy(no added warnings), fmt, WASM compile,
   browser E2E and official/new-seeded controls. Known8 official failures
   remain visible and name-compared, never omitted or called PASS.

## Implemented binding contract

`sliceProjectPlateProcessed(input: Uint8Array, plate: number | undefined,
 overridesJson?: string) -> Promise<SliceOutput>`.
Both palettes now derive from `GCodeProcessorResult`: base extruder colors,
ColorChange item colors, and gray for nonempty custom items. The caller palette
argument has been removed (breaking change; see `web/README.md`). See
[Filament ColorPrint slice](web-filament-colorprint.md). `sliceProjectPlate`
returns G-code only; no legacy palette fallback or alternate conversion lives
in the Rust adapter.

`SliceOutput.takeGcode()` returns Uint8Array. `takeMoves()` and
`takeVertices()` return owned buffer handles; their take methods empty the
corresponding buffer, so consumers take once and call `free()` on handles.

| Handle | Method | Typed array and per-record stride/order |
|---|---|---|
| MoveBuffers | takeValues | Float32Array,12: xyz,deltaE,feedrate,width,height,mm3_per_mm,travel_dist,fan,temp,time_normal |
| MoveBuffers | takeIds | Uint32Array,2: gcode_id,layer_id |
| MoveBuffers | takeTags | Uint8Array,5: core move type,core extrusion role,extruder_id,color_id,internal_only |
| VertexBuffers | takeValues | Float32Array,16: xyz,height,width,feedrate,actual_feedrate,mm3_per_mm,fan,temp,time_normal,time_stealth,layer_duration,PA,acceleration,jerk |
| VertexBuffers | takeIds | Uint32Array,2: gcode_id,layer_id |
| VertexBuffers | takeTags | Uint8Array,4: libvgcode role,move type,extruder_id,color_id |
| VertexBuffers | takeToolColors / takeColorPrintColors | Uint8Array RGB triples |
| VertexBuffers | takeLayerZs | Float32Array, one z per layer: libvgcode `Layers::get_zs` after `ViewerImpl::load` (ViewerImpl.cpp:1002-1005) updates `Layers` with every vertex |
| VertexBuffers | viewRange(minLayer, maxLayer, visibleRoles, visibleOptions) | Uint32Array `[first, last, current]`: ares-vgcode `ViewerImpl` (port of ViewerImpl.cpp `is_visible` :1713, `update_view_full_range` :1728-1815, `set_layers_view_range` :1329, `get_current_vertex_id` ViewerImpl.hpp:132) after toggling role / option visibility to the given bits. The vertices stay loaded in the `VertexBuffers` object until it is freed |

Both handles expose count; VertexBuffers also exposes spiralVaseMode.
Raw and libvgcode role ordinals differ (notably BottomSurface, Ironing,
InternalBridge, Brim), so callers must use the vocabulary of each handle.
Raw records include dummy/internal-only/Seam records. Converted vertices
exclude dummy and include phantom boundaries; counts are intentionally different.

## Test evidence

API-missing RED preceded implementation. After implementation,23 vgcode/WASM
tests pass; targeted Clippy adds no
warnings. Existing handoff color conversion matches Color.cpp332–367,
including invalid hex pairs remaining black before minimum-brightness clamp.

Real Chrome151 E2E:
Ender3:5564 moves/7347 vertices; KSR:635147 moves/823716 vertices; zero
mismatches. Native oracle actually ran on the browser's emitted G-code,
not on a native replacement slice. Full raw typed buffers, native CSV,
stdout and diff.csv remain in each fixture directory. E2E requires explicit
oracle and WASM package paths and the oracle CPU (`ORCA_REPLAY_CPU`); it cannot
silently pass without them.

Reproduce after building release ares-wasm and running wasm-bindgen --target web:

```
ARES_WASM_PKG=/absolute/bindgen/output \
ARES_MOVES_ARTIFACTS=/new/evidence/directory \
ARES_PROCGCODE_BIN=/absolute/orca-slicer \
ORCA_REPLAY_CPU=<one CPU> \
ARES_E2E_CHROME=/absolute/chrome node web/e2e/wasmMoves.mjs
```

Full workspace validation:7612 PASS,
9 existing skips; Clippy/fmt/WASM check exit0. Clippy reports102 core-lib/103
core-test/21 parity warnings, all in existing code, with no binding warnings. Full controls:seeded407/407
PASS; official399 PASS/8 accepted existing failures. Exact failure-name
comparison adds/removes no names (`failure-comparison.json`). This is not
an all-green official claim; no assertions or comparator bounds were relaxed.

The current native processor and browser expose every converted vertex's
gcode_id as u32 in takeIds(), including phantom vertices copied from curr.
The large16777217 unit case and both browser oracle projections explicitly
exercise lossless IDs. No web temporary conversion code was deleted.

Browser rendering UI integration is a separate web slice, which also removes
the temporary conversion. M854 remains a separate next slice.
