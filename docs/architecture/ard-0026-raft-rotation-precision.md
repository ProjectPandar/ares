# ARD-0026: Preserve float trig at the support filler boundary

`FillSupportBase::fill_surface` calls float trig overloads and promotes
their results for double coordinate arithmetic. The Rust port must keep
this precision boundary: nanometer endpoint differences can change
five-decimal extrusion output even when three-decimal XY output agrees.

This decision is specific to the cited caller. General point rotation and
input polygon rotation take double angles upstream and must not inherit
this narrowing. G-code comparison remains exact after generator metadata
normalization; tolerances cannot replace the source behavior.
