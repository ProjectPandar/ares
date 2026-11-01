# Honeycomb multiline boundary hooks: beta RED localization

## Source boundary

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/Fill/FillBase.cpp::Fill::connect_infill`, especially
2014–2046 (remaining-endpoint hook direction and multiline-copy clearance),
with `distance_to_open_end` at 1947–1963.
Rust destination: `crates/ares-core/src/fill/connect/apply.rs`,
`apply_remaining_endpoints` and `take_hook`. This is an upstream connector
rewrite slice, not an independently designed Ares infill algorithm.

Included in this RED: ordered raw XY/E extrusion records of the first Sparse
layer (sixth layer, Z=1.2), runtime Native double replay versus public
`slice_project_plate_validated`. No Native G-code is checked in. No numeric
normalization, sorting, or weakening of the complete-output comparator.
Deferred: other layers, cooling/M73, travel/wipe, sorted-arc optimization,
join threshold, other patterns, and any claim of complete-output parity.
The existing connector is a temporary incomplete compatibility shell around
`Fill::connect_infill`; the RED does not authorize production GREEN.

## Input and first divergence

Frozen SHA-256:
`e1d22463fc2b151ed1043f837c2833432c6ce3724a12361bbb011fef1b8ca0c3`.
This is the existing qualification honeycomb input derived from ledger
`case-8gDVxY`, with cooling disabled, not a new geometry search. Multiline=2.
The historic ledger observed a rounded Ares path as well; do not treat its
historic bytes as identical to the current baseline. Current artifacts give
the independently reproduced endpoint residual below.

First Sparse path starts at Native `(114.020,110.683)` versus Ares
`(114.020,110.901)`. Ares emits an extra
`G1 X114.02 Y110.683 E.00723`; the following eleven extrusion records match
Native exactly. The far endpoint extends from `(111.945,105.980)` to
Native `(111.790,105.980)` versus Ares `(113.573,105.980)`.
These are boundary hook differences, not evidence of a changed honeycomb
main-chain angle or phase.

Temporary observation-only `take_hook` instrumentation confirms the exact
integer mutations (local mm origin is machine XY 110,110):

- trace line 7: `(4019693,683179)` gains `(4019693,901078)`;
- trace line 8: `(1944930,-4019693)` gains `(3573248,-4019693)`;
- anchor length 1628318.548 scaled units; half width 203539.819.

These reproduce both discrepant G-code endpoints after translation and
rounding. Instrumented public output SHA equals baseline output SHA
`e58f4be139ac191581bc00aa590fdaad468331519046bb5d73637ab48d9e7881`.
Instrumentation was restored. The beta source's nearest-open-end direction
and multiline sine-clearance cap are absent from the Rust branch. This
localizes the owning block; it does not yet prove which subexpression alone
closes the RED. No experimental GREEN was made.

## Approved GREEN slice

The remaining-endpoint pass now tracks open ends separately from consumed
intersections, searches up to 64 ends in each direction, and applies the beta
multiline sine-clearance cap using `path_length_along_contour_ccw`.
The private `connect/hooks.rs` module holds the source-owned lookup and cap;
`apply.rs` retains mutation order and endpoint merging. Join thresholds and
sorted-arc selection are unchanged.

The scoped layer-six witness closes exactly: 22 ordered extrusion records
match runtime Native bytes. The existing Hilbert surface witness passes before
and after. The line anchoring witness retains its baseline complete-output
M73 placement failure; its API and CLI output bytes are unchanged. A third
rectilinear sample cannot load repository-relative profiles and supplies no
API parity result. No complete-output parity claim or FULL run is made.

External evidence retained outside the repository.
