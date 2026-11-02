# Beta 3D honeycomb public geometry RED

Authority: OrcaSlicer 2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`, `src/libslic3r/Fill/Fill3DHoneycomb.cpp:336–436` and its grid, top-cover and smoothing dependencies. The implementation base is `f4ea41332cdcb7eadf9adbc7f7c1936c0ad7e9c5`.

The boundary and acceptance criteria are defined in [the rewrite specification](2026-11-02-beta-3d-honeycomb-rewrite-spec.md). No product implementation is included in this RED.

## Public seam and retained inputs

The tests call `PlaterSession::load_with_profiles` and `slice_plater_with_profiles` using the existing `tests/parity/cube10.stl`, raw vendor catalog and LH Stinger preset selection. The moved case also calls `translate_instance` with `[20,-15,3]`. Native receives the same flattened presets and independently positioned cube, as in the existing strict test.

The ordinary centered and moved witnesses do not change any process settings. The other witnesses apply the same explicit option overrides to the public Rust request and the Native process preset.

Each artifact directory retains the input STL, positioned Native STL, catalog, selection, overrides, flattened presets, raw Native/API G-code and command/exit receipts. SHA256 values are recorded for both input forms, raw streams and derived geometry streams. Derived stream records retain original line, Z and role context in separate JSON files.

## Scoped assertion

The geometry assertion keeps ordered `G0/G1/G2/G3` command and operand tokens, including extrusion, without numeric parsing or rounding. It removes feedrate operands, pure feedrate-setting commands and non-geometric metadata. Coordinate/extrusion mode commands `G20/G21/G90/G91/G92/M82/M83` remain ordered. Every retained record is compared with the existing byte-difference renderer.

This is a new geometry-only assertion, not a replacement full-stream comparator. The existing strict plater assertion is unchanged. Its whole-stream result is also recorded for every new case. Feedrate closure remains a separate mandatory check within the geometry slice's exit criteria.

## Observed RED

With the required official Native and profile-entry reference gates enabled, nextest selects seven tests. All seven fail the geometry assertion, with zero selected skips. The run filters out 2178 other tests. Every Native command exits zero and every API call returns G-code.

All first differences occur at retained geometry record 362, original source line 568, Z=1.0, in the travel into sparse infill.

| Witness | Override | Native first record | Rust first record |
| --- | --- | --- | --- |
| Centered | none | `G1 X122.139 Y134.924` | `G1 X122.139 Y134.046` |
| Moved | none | `G1 X142.139 Y119.924` | `G1 X142.139 Y119.046` |
| Dense | density 60% | `G1 X122.139 Y141.423` | `G1 X114.417 Y142.139` |
| Multiline | count 3 | `G1 X122.139 Y141.355` | `G1 X113.861 Y141.545` |
| Complete top | enabled | `G1 X122.139 Y134.924` | `G1 X122.139 Y134.046` |
| Open top | disabled | `G1 X122.139 Y134.924` | `G1 X122.139 Y134.046` |
| Smoothed | factor 35% | `G1 X122.139 Y134.924` | `G1 X122.139 Y134.046` |

The Native complete-top and smoothing cases produce geometry streams different from the open-top control. Their effective footer options agree with the supplied overrides. The first mismatch happens before those later branch-specific differences. No whole-stream equality or product correctness is claimed from these failures.

The original strict test separately selects one FAIL with 2177 tests filtered out on the implementation base. It still fails at normalized byte 2157, line 91, at the M73 insertion difference. It has not been weakened or relabeled as a geometry PASS.

## Qualification limits

The ordinary-density and over-42%-density paths have public REDs. This matrix does not independently establish coverage of the low-density small-module condition or combined-infill thickness behavior. Those conditions require their own qualified public witnesses before claiming coverage. Reusing a branch's first XY mismatch does not prove every internal branch ran.

The frozen existing honeycomb/fill family remains 117 tests. These new tests are additional witnesses, not substitutions for that regression inventory. Product code remains blocked on review of this RED and its actual first records.
