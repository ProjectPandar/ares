# ARD-0066: Top-shell scalar range parity investigation

Upstream owner: `PrintConfig.cpp:6730-6739` declares top_shell_layers as coInt,
min 0, with the scalar representable ceiling INT_MAX (`Config.hpp:955-983`,
and ConfigOptionDef's larger default ceiling at :2475-2479).
`PrintObject::discover_vertical_shells` (`PrintObject.cpp:2223-2250`) projects
top regions from higher layers. `discover_horizontal_shells` (:3983-4020)
propagates shell surfaces in the other direction. Rust destinations already
include `prepare_infill/vertical_shell_projection/gather.rs::gather_top`
and `horizontal_shell_propagation/window.rs`; native fixture scaffolding is
a temporary adapter to the existing config and byte-output boundaries.

Include exact 0 / INT_MAX / fixed-seed full-range interior in the domain,
original export application, final config and complete ordered output tests.
Preserve all successful and divergent paired artifacts. Keep the normal,
non-spiral fixture and its default thickness behavior. Do not reduce the
requested count to the number of model layers.

The upstream `int itop = int(idx_layer) + n_top_layers` (:2227) overflows
at INT_MAX for nonzero indices. Native debugging confirms the original binary
performs a 32-bit addition and sign-extends its result before window comparison:
at index 1 and count 2147483647 the count-end is -2147483648. Repeated identical
inputs retain identical original motion. Rust's previous i64 addition instead
projects top shells through the whole fixture, diverging in complete output.
C++ signed overflow is language-level undefined behavior: this is not a
portable C++ semantics claim. The measured output contract belongs to the
selected official OrcaSlicer 2.4.2 binary, SHA256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Other compiler builds need their own evidence before parity can be claimed.

Replace only this count-end expression with explicit i32 `wrapping_add`, then
widen its result for the existing comparison. This gives Rust defined arithmetic
for the observed native rewrite contract, with no runtime fallback or altered
request. Do not generalize wrapping to unrelated computations or reproduce
uninitialized/random values. Validate INT_MAX-1 and INT_MAX-100 as adjacent
overflow/non-overflow controls in addition to min/max/full-range seed.

Defer spiral mode, thickness/multi-region cross-products and other implicit
ranges. Retain full comparison without new filters, fallback or retries.
