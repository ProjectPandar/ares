# AlignedBack occlusion object-affine witness

Source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`SeamPlacer.cpp:630-671::compute_global_occlusion`,
`Print.hpp:325-328::trafo_centered`, and
`deps_src/admesh/stl.h:362-369::its_transform<double>`.
See [the approved object-affine boundary](../../../docs/architecture/seam-occlusion-object-affine.md).

`x1c-thin-0.3mf` SHA256:
`eb7bdc6e74029dfa25da954c231142f59eb37da648943a425516f1eb54dd894e`.
This is the unchanged accepted input, not a regenerated model or Native golden.
Its model comes from the already committed
`tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r0.3mf`:
32 vertices/60 triangles, closed opposite edge pairs, positive78.00000312mm³.
Every other archive member, including build/part transforms and stored object
aligned_back override, is byte-identical to that source. Only the global
project config is conditioned to an official-exported X1 Carbon stock profile:
classic, ccw, detect_thin_wall0,2 walls,0.2mm layers. This is not a one-key
stock change from the original Creality fixture. SHA256SUMS.json records it.

The source double object translation Z0.300000012 is nonidentity. Promoting
the stored float vertex before the double affine leaves bottom Z
7.907102750337458e-11; narrowing matrix coefficients first leaves0. That
changes sampled point/KD traversal order and rounded visibility scores;
the first-layer right inner loop starts at different seam vertices.
The test has no expected coordinate/index/timing constants.

Test seam: unchanged `runner::native_stream_case::check` with Native2 FIRST,
actual public validated byte API and real CLI, raw API==CLI, complete ordered
`golden::compare_ordered_bytes_generator_only`. On RED, strict comparison must
actually fail in each Native mode; there is no expected-failure assertion or
mask/tolerance. The helper writes both outputs before the RED panic, so raw
files are independently audited without claiming its final raw assertion ran.
All Native outputs/commands/cache receipts/comparisons stay outside Git.
