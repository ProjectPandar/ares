# Scarf default-speed threshold fixtures

Source slice: OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1,
`GCode.cpp:5808-5813,6515-6526`; approved spec
`docs/architecture/gcode-scarf-default-speed-threshold.md`.

`boundary.3mf` is a public byte-input Creality Ender-3 classic one-wall scarf
case, frozen before Native comparison. It derives from the earlier square
scarf input SHA7e08b24c0cb9b3afb614fe7a9ab7e8478e22fa0c37ca60b3bc79a1fd21a34d4e.
Only the object mesh and project-settings ZIP member were changed: XY half
5.20013714 (binary32 value5.200137138366699), threshold6.3663721574934415,
scarf50%, ramp0. Z, other mesh/topology/settings members remain unchanged.
Cooling and volumetric speed were already disabled in the original input.
This is an intentional new input, not reference-output or comparison tuning.

A source-unchanged public CLI slice under nix-gdb on Rust maindbcd52d5a
observed the actual second-layer original loop as one ExternalPerimeter path:
`(-5000137,-5000137,0) -> (5000137,-5000137,0) ->
(5000137,5000137,0) -> (-5000137,5000137,0) -> close`.
Actual caller Option is None; actual scarf_speed returns36; features consumes
that slope speed with requested=None and returns36. This is not a scalar-only
or computed-but-masked claim.

The measured scaled length40001096 exceeds the scaled limit40001095.99999999;
the per-member XY-mm length40.001096 equals the local limit40.001096. Native
therefore leaves caller speed default and caps scarf to min(120,50%*120)=60.

The first fresh CPU0/seed0, single-attempt CLI E2E captured a real complete-byte
RED: Native non-first outer-wall F3600/F7200, Ares F2160/F7200. Only generated-by
identity was normalized; M73/footer retained. First byte difference1059/line42
is M73 P57 versus P56. No unrelated time/progress repair is authorized.

## Frozen controls

Before any Native comparison of these controls, clone boundary bytes and
change only project-settings threshold to6, plus the stated scarf speed.
These ordinary non-small loops exercise the default cap. The existing
`orca_parity_small_loop_true_loop_scarf_ramp` remains the positive caller
small-loop/ramp control; its expectations are not changed.

| Fixture | SHA256 | Scarf | Expected non-first outer F |
|---|---|---|---|
| boundary.3mf | 1646ba992cc58685074abd95e009d17f85f88e190379912201d9d9f88399f4e5 | 50% | 3600/7200 |
| percentage.3mf | 88da8bb4273865225bf38359379a14d1947c83cd71e51a05076d5c4cfcc78782 | 50% | 3600/7200 |
| absolute.3mf | 2e982da9e5eb5cadc889996a2579196027c8baa4b47bcc8cd414579f81954ba6 | 60 | 3600/7200 |
| above-role.3mf | 3884f95ff09b00210eab4c06c772a16b5dfa64ae231a766a0a42301132e75366 | 150% | 7200 |

Rust E2Es in `crates/ares-cli/tests/orca_parity/runner/scarf_default_threshold.rs`
call the public project byte slicing API through the existing full-byte
parity runner and retain per-case speed-witness JSON, input/output streams,
manifest and Native receipt. Run with a configured Native parity environment:

```
cargo nextest run -p ares-cli --test orca_parity -E 'test(scarf_default_threshold)'
```

The repeatable first-call artifacts are retained under
`${TMP_ROOT}/ares-scarf-threshold/post-loop-3d65/` (input audit, gdb log reference,
reachability observation, frozen-control declarations and fresh-native-first).
No panic probe was applied or committed; OrcaSlicer was not rebuilt.
