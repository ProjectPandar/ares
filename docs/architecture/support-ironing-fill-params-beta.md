# Support-ironing fill parameters and the non-finite 3D-honeycomb domain

Status: proposal for review. This record does not authorize production changes
or a RED fixture. The baseline is
`f8d715c4c2303b09fed72260d23997733221f2e4`; upstream is Orca beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Decision proposed for review

Keep two contracts distinct:

1. **Defined adapter behavior:** ordinary contact ironing passes the upstream
   `FillParams::layer_height` default of zero. It does not substitute the
   physical support-layer height. Physical layer height and extrusion-flow
   height retain their existing meanings and values.
2. **Undefined upstream kernel domain:** do not reproduce a non-finite
   floating-to-integer conversion. For 3D honeycomb, propose checking the
   computed Z scale before constructing the grid request or entering
   `makeGrid`. If it is non-finite, return an empty polyline set for that fill
   component. This is an explicit Ares-defined behavior where upstream has no
   defined result, not a claim of Native geometry parity.

The second decision is an ARD-only proposal awaiting approval. It must not
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
- The infinite grid Z reaches `makeGrid` at line 305, `makeZigZag` at
  line 264, `getCriticalPoints` at line 79, and `triWave` at lines 43–44.
  `triWave` converts its non-finite floating value to `int`. Such a value
  cannot be represented in the destination integer type; this is undefined
  behavior under C++ [conv.fpint]. IEEE-defined division by zero does not
  make the subsequent floating-to-integer conversion defined.

Contact-ironing density is fixed by the caller. Changing sparse-infill
percentage, ironing flow, or ironing spacing cannot select the low-density
branch for this call. A generic low-density 3D-honeycomb experiment therefore
cannot stand in for a discriminating contact-ironing adapter RED.

The proposed guard is on **computed non-finite Z scale**, not simply on zero
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
  `fill_component`: if the proposed undefined-domain policy is approved,
  propagate the non-finite-Z-scale result as an empty component before
  `grid::make_grid`. No non-finite grid coordinates should be created merely
  to be converted or clamped later.

The current `SurfaceRequest` adaptation is a temporary compatibility shell
around the cited `SupportCommon` / `FillParams` concepts. This slice separates
those upstream call-specific parameters from physical layer data; it does not
introduce an Ares-owned slicing pipeline or a new public configuration API.

Coordinate the bounded guard with the geometry owner. Do not rewrite
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
not PASS and not a FULL OPEN. After review, remove that leg from the eligible
Native-comparison registry in
`crates/ares-cli/tests/orca_parity/option_coverage/support_ironing_pattern_domain.rs`
and place a citation to this record beside the excluded class's explanation.
Do not use an ignored test or a passing comparison exception. Retain historical
artifacts; do not rewrite them as GREEN evidence.

List the eligibility change explicitly in the next inventory. Compare the
same eligible inventory before and after a later port, and report the UB
reclassification separately: removing an undefined case is not a FAIL-to-PASS
transition or a new whole-case PASS.

## Defined-domain adapter RED gate

The adapter RED must use a different, reachable support-ironing pattern that
consumes fill-layer height and has a fully defined zero-height domain. Its
selection must follow the geometry-owner static recommendation. That concrete
recommendation and fixture qualification are **pending**; no pattern or fixture
is approved by this proposal.

A concentric/Arachne path is a candidate, not an established answer. Confirm
actual factory dispatch and helper defaults: `FillParams::use_arachne` defaults
to false, so a concentric name alone does not establish that the height-consuming
Arachne branch is reached. Audit the selected branch's zero-height calculations
and conversions before executing it as a Native oracle. If no defined,
observable adapter case is found, return for review rather than adding a
nondiscriminating test or a new diagnostic API.

Before writing the RED, name its existing public byte-API / CLI seam, selected
pattern, source consumer, defined-domain argument, and observable mismatch.
Qualify runtime Native double replay and API/CLI identity. The new test must
fail on this baseline for the adapter discrepancy, not for a different default
or an unrelated geometry residual. Preserve complete outputs and the unchanged
complete comparator, including feeds, M73, metadata and footer, with only the
established generator-comment exception.

Any later test of the proposed empty-component policy is an **Ares-defined
behavior contract**, not a Native-parity test for the UB class. Keep that
classification separate from FULL Native comparison counts. Do not embed
captured non-finite values, observed coordinates, or historical output bytes as
expected results for the undefined path.

## Review and implementation gates

Send this ARD first. Obtain approval of the empty-component policy and the
concrete defined-domain RED recommendation before test or production changes.
Deliver the qualified RED separately and obtain review before implementation.
A later port must preserve finite-domain behavior, use an identical eligible
paired inventory with zero PASS-to-FAIL transitions, retain verifiable artifacts,
and pass rustfmt and strict Clippy. Rust tests use `cargo nextest run`.

Deferred: all other fill/kernel changes, polygon preparation, layer ownership,
flow, ordering, organic support, timing/progress arithmetic, filament maps and
rematerialization, E3D preset production, preset refresh, and footer work. The
whole-variant model-time discrepancy remains a separate rewrite slice.
