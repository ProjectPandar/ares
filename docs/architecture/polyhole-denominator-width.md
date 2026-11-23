# Match the upstream polyhole denominator width

## Source boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/PrintObject.cpp:226`, computes the circumscribed radius in
`create_polyholes` as `radius / float(std::cos(PI / nb_edges))`. `PI` is a
double constant (`libslic3r.h:81`). The angle division and cosine are double;
the cosine is narrowed to float before the float radius division.

In `crates/ares-core/src/project_slice/hole_to_polyhole/polyhole.rs:46`,
`create_polyholes` currently divides a float PI by a float edge count and
uses a float cosine. Replace only that denominator with the double angle
and cosine, narrowed once before the existing float division. Do not change
edge selection or capping, detection and grouping, point construction,
angle sampling (`PrintObject.cpp:227–229`), zero handling, or toolpath
formatting. No new math abstraction or target-specific branch is needed.

For 42 edges, on native x86-64 glibc 2.44 the two denominator bit patterns
are `3f7f48bf` (current) and `3f7f48c0` (upstream width). The current value is
smaller, so the radius is larger for the affected numerators. For example,
with a scaled hole radius of 10,375,085, the rounded X-axis vertex relative
to the center is 10,404,178 rather than 10,404,177. That isolated calculation
is not a G-code expectation.

## Public CLI witness

Use a synthetic Ender-3 project with one 30 × 30 × 1.2 mm block and a vertical
64-sided hole centered at (15, 15) in the block. The input hole radius is
10.450775 mm with a 0.4 mm nozzle, which yields 42 output sides. The build
translation is (110.000775, 110, 0) mm. Enable the polyhole option on the
part, turn twist off, and use classic single walls and 0.2 mm layers. Set
hole and elephant-foot compensation, infill, solid shells, and arc fitting
to zero. The Native edge cap of 50 does not bind. Derive the fixture from
the existing synthetic `tests/parity/hole-to-polyhole/block-hole.3mf`
connectivity; no output captured from Native is part of the fixture.

On the unchanged implementation, the public CLI and OrcaRunner outputs
match in length and differ by one digit after excluding the generator line
with the existing comparator. Native emits `G1 X132.838 Y117.729 E.0491`;
Ares emits `G1 X132.838 Y117.728 E.0491`. This is on the lower-Y side of the
hole, consistent with an increased radius in Ares. The minimal expression
change must eliminate that difference without changing any other compared
bytes; otherwise the causal attribution is not established.

## Test-first verification

1. Add the synthetic project and a public CLI parity case under
   `option_coverage::hole_to_polyhole`. Use the existing `OrcaRunner` to
   slice Native at runtime and the public `ares slice` CLI on the same
   project. Require a successful Native slice and the expected Native move
   to establish that the fixture exercises this path. Assert ordered byte
   equality with the existing generator-only normalization; do not assert
   a particular incorrect Ares value or commit a Native output.
2. Run the case on the unchanged implementation. Confirm the failure is the
   observed coordinate digit, not fixture loading or supplier setup.
3. Change only the denominator width. Require the same case to pass by exact
   comparison. Run the affected polyhole cases paired in official and seeded
   modes with the canonical reference environment and unchanged comparators.
   Run formatting, workspace all-target Clippy and the WASM build check.
