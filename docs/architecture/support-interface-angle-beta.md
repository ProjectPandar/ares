# Normal support-interface angle selector (beta)

Port the normal support-interface selector and its caller from Orca beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`, not a timing workaround.
The public RED baseline is `8612802fec12baeaa32600edb8e0f3795d3eeb95`.
This record authorizes an ARD/RED boundary, not a production port.

## Upstream owner and Rust destination

- `src/libslic3r/Support/SupportParameters.hpp:308–329`,
  `SupportParameters::support_interface_angle`: select the ordinary interface
  angle from interface pattern, resolved support style, and interface ID.
- `src/libslic3r/Support/SupportCommon.cpp:1729,1852–1880`,
  `generate_support_toolpaths`: use that angle for ordinary interfaces; keep
  actual raft-contact and interface-as-base choices distinct.
- `src/libslic3r/Fill/FillRectilinear.cpp:3490–3501` is the downstream live
  input witness, not a fill-kernel rewrite boundary.
- Destination: `crates/ares-core/src/project_slice/support_material/`
  `toolpath_parameters.rs` for the source-local selector, and `toolpaths.rs`
  for the normal-interface caller currently at lines 49–63. Existing
  `ObjectOptions` supplies pattern/style; do not duplicate configuration state.

The current Rust shortcut to `raft_interface_angle` is a temporary compatibility
shell around the upstream `SupportParameters`/`SupportCommon` concept. Replace
that shortcut with the cited normal selector, not an Ares-owned angle policy.
Keep core independent of filesystem, terminal, viewer, and platform runtime APIs.

## Qualified boundary

The frozen public input is
`tests/parity/skirt-support-occupied-input/support-overhang.3mf`, SHA-256
`58bf8c9a7a28cc19d7ee61c45458ae319dbce6882036cc7a7f6dde06d60c288d`.
At support layer index 46, interface ID 0, print Z9.4:

- Both live configurations select Organic style and Auto interface pattern.
  Both have base angle 0, interface angle `1.5707963705062866`, and raft
  interface angle 0.
- Native's selected angle and actual fill angle are `1.5707963705062866`;
  Rust's selected angle and actual rectilinear spec angle are
  `0.7853981852531433`.
- The two post-safety-offset input ExPolygons have 16 contour vertices and no
  holes. Polygon, contour, hole, and vertex order match exactly as integers.
  Observations used Native's live `Surface::expolygon` argument and Rust's
  borrowed `ExPolygon`, in actual fill-call order, before internal fill inset.
- Observed width, height, spacing, nozzle diameter, and density match. Native's
  captured bounding-box center, derived as `(0, 0)`, matches Rust's reference.

Native's original vector temporary and two boolean locals were unavailable;
no guessed addresses or inferior calls recovered them. The actual live owner
fields above qualify the polygon/angle boundary, not complete fill-state
identity. The approach at raw line 34769 is at hop Z9.8, not print Z9.8:
Native `G1 X104.101 Y115.783 Z9.8`, Rust `G1 X104.353 Y104.592 Z9.8`.
The following Support interface deposition is vertical versus diagonal.

The observed total-time difference is downstream of unequal motion inputs.
Neither this witness nor the separate early one-ULP elapsed-cache difference
proves sole causality for the full time delta. Do not repair M73 arithmetic,
force a remaining-minute value, or claim the complete case is already GREEN.

## Included literal behavior

Port the entire small normal selector switch, only through this caller:

- Rectilinear: subtract `float(M_PI_4)` from the interface angle for Snug;
  otherwise retain the interface angle.
- RectilinearInterlaced: add positive `float(M_PI_4)` for odd interface IDs,
  negative for even IDs, to the interface angle.
- Grid: use the base angle.
- Remaining supported interface patterns: use the interface angle.

Preserve f32 operands, signed parity, and parentheses. Use existing typed Rust
configuration variants exhaustively. Do not normalize angles or reuse the
opposite-sign raft parity helper. Downstream cached ordinary-interface and
ironing input angles follow this caller; no independent ironing change belongs
in this slice.

## Deferred behavior

Do not change actual raft-angle selection, interface-as-base routing, polygon
preparation, geometric ownership, fill kernels, flow, path sorting, or ironing
algorithms. Organic roof sampling, graph/draw/skirt behavior, time estimation,
progress formatting, filament grouping/maps, preset refresh, and footer work
remain separate. Do not extend this into a whole support pipeline rewrite or
attribute every timing difference to this selector.

## Public RED and acceptance gate

Use the frozen bytes through runtime Native double replay, the public byte API,
and CLI. A focused test compares the first Support interface XY deposition
command with Native; it must fail on the current diagonal/vertical mismatch.
Save complete API/CLI outputs and both complete ordered comparison results even
in that focused test. An independent whole-output sibling must retain the
unchanged complete comparator and its current M73 failure.

A focused command assertion is not a whole-case verdict. Preserve feeds,
progress, metadata, footer, and all remaining bytes, with only the established
generator-comment exception. No sorting, filtering, ignored assertion, forced
value, or new comparator exception can produce a whole-case GREEN.

Deliver qualified RED and this source boundary before any production port.
A later authorized port requires the identical paired inventory, zero
PASS-to-FAIL transitions, reproducible artifacts, rustfmt, and strict Clippy.
