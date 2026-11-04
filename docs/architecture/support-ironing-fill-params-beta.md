# Support-ironing fill parameters and the non-finite 3D-honeycomb domain

Status: Accepted. Upstream is Orca beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Decision

Keep two contracts distinct:

1. **Defined adapter behavior:** ordinary contact ironing passes the upstream
   `FillParams::layer_height` default of zero. It does not substitute the
   physical support-layer height. Physical layer height and extrusion-flow
   height retain their existing meanings and values.
2. **Undefined upstream kernel domain:** do not reproduce a non-finite
   floating-to-integer conversion. For 3D honeycomb, check the
   computed Z scale before constructing the grid request or entering
   `makeGrid`. If it is non-finite, return an empty polyline set for that fill
   component. This is an explicit Ares-defined behavior where upstream has no
   defined result, not a claim of Native geometry parity.

The second decision is an Ares-defined domain policy. It must not
become a physical-height fallback, a numeric clamp, a fixture-specific output
replacement, an ignored assertion, or a timing correction.

## Source boundary and confirmed UB chain

The owning upstream files and functions are:

- `src/libslic3r/Support/SupportCommon.cpp:2037–2070`,
  `generate_support_toolpaths`: create the contact-ironing filler, set its
  print Z, angle and spacing, and call the fill wrapper with density one.
- `SupportCommon.cpp:610–621`, `fill_expolygons_generate_paths`: construct
  `FillParams`, set density and `dont_adjust`, and leave `layer_height` at its
  defined default.
- `src/libslic3r/Fill/FillBase.hpp:91`, `FillParams::layer_height`: the default
  is zero. This is initialized source state, not an uninitialized-value case.
- `src/libslic3r/Fill/Fill3DHoneycomb.cpp:371–393`,
  `Fill3DHoneycomb::_fill_surface_single`: scale that height and compute the
  grid's Z scale. Density one necessarily selects the dense branch at
  lines 374–380; its zero denominator produces positive infinity.
- The call to `makeGrid` is at line 391, with Z arguments at lines 392–393.
  Inside `makeGrid`, line 305 calls `makeZigZag`; line 264 inside `makeZigZag`
  calls `getCriticalPoints`, which calls `triWave` at line 79. `triWave` is
  defined at lines 43–44.
  `triWave` converts its non-finite floating value to `int`. Such a value
  cannot be represented in the destination integer type; this is undefined
  behavior under C++ [conv.fpint]. IEEE-defined division by zero does not
  make the subsequent floating-to-integer conversion defined.

Contact-ironing density is fixed by the caller. Changing sparse-infill
percentage, ironing flow, or ironing spacing cannot select the low-density
branch for this call. A generic low-density 3D-honeycomb experiment therefore
cannot stand in for a discriminating contact-ironing adapter RED.

The guard is on **computed non-finite Z scale**, not simply on zero
layer height. In the generic low-density path, zero height can leave the
initial Z scale finite even though `layersPerModule` becomes infinite. Do not
reject or empty that path merely because an intermediate is infinite; its
reachable conversions require their own defined-domain qualification.

## Rust destination and compatibility boundary

- `crates/ares-core/src/project_slice/support_material/ironing.rs`,
  `IroningSurface::append_paths`: adapt the contact-ironing request to the
  defined upstream default instead of forwarding `input.height` as the fill
  parameter. Do not alter the physical layer or the ironing extrusion flow.
- `crates/ares-core/src/project_slice/fill_entities/three_d_honeycomb.rs`,
  `polylines`: preserve direct forwarding of the selected fill-layer-height
  argument into the kernel parameters. Do not globally zero object-infill
  requests; upstream `Fill.cpp` sets their physical height separately.
- `crates/ares-core/src/fill/three_d_honeycomb.rs`, `grid_request` and
  `fill_component`: implement the undefined-domain policy and
  propagate the non-finite-Z-scale result as an empty component before
  `grid::make_grid`. No non-finite grid coordinates should be created merely
  to be converted or clamped later.

The current `SurfaceRequest` adaptation is a temporary compatibility shell
around the cited `SupportCommon` / `FillParams` concepts. This slice separates
those upstream call-specific parameters from physical layer data; it does not
introduce an Ares-owned slicing pipeline or a new public configuration API.

Keep the guard bounded. Do not rewrite
`grid.rs`, waveform construction, tops, smoothing, connection, clipping,
rotation, or integer-conversion algorithms. Defined finite-Z-scale behavior
must remain unchanged on WASM, Windows, macOS, and Linux.

## Historical observation and UB reclassification

The previously frozen input with SHA-256
`195d59c8bc339fa72d2b7959575185c3cbfa0bd3ce3a9c6b1a3562041f73ad1e`
was observed at support layer 48 and print Z9.8. Native's fill-height parameter
was zero while its physical layer height, and Rust's actual backend height
argument, were approximately 0.2. Two ordered pre-inset polygons matched with
16 contour vertices and no holes. The old complete comparison first differed
at byte 503, line 19: Native reported M73 R35 and Rust R32. These observations
explain how the parameter boundary was found; they are not golden values or
an oracle for the undefined kernel domain.

Native's computed Z-scale local and Rust's kernel-parameter aggregate were
optimized out in the captures. The UB conclusion is the static source chain
above, not a claim that a debugger measured the intermediate infinity. Stable
Native output across replays does not turn undefined behavior into a portable
specification. The earlier complete-output comparisons are historical
observations only; they cannot support a Native comparison claim for this
class.

Reclassify the existing
`support_ironing_pattern_three_d_honeycomb` Native-parity leg as UB-derived,
not PASS and not an open full-parity case. Remove that leg from the eligible
Native-comparison registry in
`crates/ares-cli/tests/orca_parity/option_coverage/support_ironing_pattern_domain/legs.rs`
and place a citation to this record beside the excluded class's explanation.
Do not use an ignored test or a passing comparison exception. Retain historical
artifacts; do not rewrite them as GREEN evidence.

List the eligibility change explicitly in the next inventory. Compare the
same eligible inventory before and after a later port, and report the UB
reclassification separately: removing an undefined case is not a FAIL-to-PASS
transition or a new whole-case PASS.

## Public Ares-defined RED and concentric controls

A static audit found no safe height-consuming pattern under
these support-ironing defaults: density one, `use_arachne` false, and classic
`fill_surface`. The adapter's zero-versus-physical-height change is therefore
not independently observable on a safe Native pattern. This supersedes the
original request for a discriminating adapter-only Native RED. Do not invent
one or add a diagnostic API to manufacture observability.

The RED instead exercises `slice_project_plate_validated` through a
separate Ares-defined integration-test binary. Use the frozen input identified
above with its support-ironing 3D-honeycomb selection. Ordinary object ironing
and tower ironing are disabled in that input, so Ironing-role paths and
extrusion belong to the support-ironing component. The defined contract is
that this component emits no ironing paths or extrusion. Verify nonempty
support/interface output and preserve complete API replay outputs, input
identity, and the contract observation before asserting the empty result.

This test must fail on the RED baseline and pass after the port.
Its expected empty component
comes from the defined-domain policy, not from Native's undefined
output. Do not invoke Native for this input or count the test as Native parity.
Do not embed captured non-finite values, observed coordinates, or historical
output bytes as expected results.

Use concentric raft and no-raft variants of the
`support_ironing_raft_offset` family as the negative controls. Classic concentric is defined under these defaults and does not
observe the layer-height adapter difference; these are preservation controls,
not a claimed adapter RED. Qualify runtime Native double replay and complete
API/CLI identity for those controls. Preserve feeds, M73, metadata and footer,
with only the established generator-comment exception. Their complete Ares
outputs must remain identical across the later port.

Report the Ares-defined RED separately from the eligible Native comparisons.
The old 3D-honeycomb Native leg is excluded by the UB classification during RED
qualification and removed from the registry in the later GREEN slice.
Report that inventory delta separately, never as a FAIL-to-PASS improvement.

## Implementation gates

Keep implementation bounded to the cited UB chain and non-finite-Z-scale guard.
No other geometry changes are in scope. The port must preserve finite-domain behavior, use an identical eligible
paired inventory with zero PASS-to-FAIL transitions, retain verifiable artifacts,
and pass rustfmt and strict Clippy. Rust tests use `cargo nextest run`.

Deferred: all other fill/kernel changes, polygon preparation, layer ownership,
flow, ordering, organic support, timing/progress arithmetic, filament maps and
rematerialization, E3D preset production, preset refresh, and footer work. The
whole-variant model-time discrepancy remains a separate rewrite slice.
