# Type1 extra-rib-length segment-unscale RED

Source pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Baseline: merged `bdb47cefe5d7851ff0e17e12d40425a1fd43bcfa`.

`rib3.3mf` is the existing `enabled.3mf` with only the decoded project setting
`wipe_tower_extra_rib_length` changed from `0` to `3` mm. All other archive
entry payloads are unchanged. Target SHA256:
`986e7ea38d1f60ff26e086c0ae8cdb4ba68336ba5d442689276deba603768f5d`.
The existing `bbl_type1_prime_tower_waypoints_enabled` is the unchanged zero-extra-rib control.

Use the existing Native-first API/CLI complete-stream comparison with no new
normalization or masking:

```
cargo nextest run -p ares-cli --test orca_parity --no-fail-fast \
  -E 'test(bbl_type1_prime_tower_waypoints_rib3_segment_unscale) | test(bbl_type1_prime_tower_waypoints_enabled)'
```

Run via the coordinator official/seeded environment, CPU11 and threads1.
Expected baseline: target FAIL, enabled control PASS in each mode. Native
source, not a literal coordinate assertion, supplies the expected full output.

## Trace and owning boundary

`WipeTowerWriter::generate_path` (`WipeTower.cpp:1124–1128`) and polygon
extrusion (`883–894`) create Segment endpoints with `unscaled<float>(Point)`.
`Point.hpp:688–705` casts integer coordinates to float before multiplying by
float(SCALING_FACTOR). Ares `type1/writer/path.rs::append_segments` instead
unscales in f64 and then casts to f32.

Native simplifies scaled integer points; Ares simplifies unscaled f64 points.
Debugger capture of the first12 path calls finds matching ordered integer
vertices both before and after simplification (Ares f64 values rescaled for
this diagnostic only). This includes the implicated fourth path's61 fitted
points: index17 is `(23615720,20220500)` in both. The live Native writer receives
Y20.220499038696289; Ares receives20.220500946044922. These equal the respective
cast/multiply and unscale/cast results, formatting to20.220 vs20.221 locally,
and then Y241.273 vs241.274 in the complete output. The shared fitter is not
the cause of this witness; this is not a claim of universal fitter parity.

Evidence: `/home/indexyz/ares-coord/evidence/w5-rib3-trace/`, including raw
before/after integer/f64 arrays, debugger scripts and live writer values.
Earlier Native-first exports are in `w5-type1-rib-length-discovery/`.

This commit is test-only RED. No production conversion, shared geometry,
formatter, comparator or registry changes. GREEN requires further approval.
