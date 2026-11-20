# Arachne proximity plate

The three synthetic projects use the same closed triangulated 16 × 13 × 1.8 mm
plate. The base box spans `[-8, 8] × [-6.5, 6.5]` in XY. Two through-holes
occupy `[-4.7, -0.83] × [-3.2, 0.75]` and `[2.1, 6] × [-1.36, 2.29]`.
Two slots reach the plate edges: `[4.26, 4.72] × [-6.5, -1.81]` and
`[-8, -4.2] × [3.42, 3.97]`. The first slot leaves a 0.45 mm ligament
beside the second hole. The Designer field is empty.

All projects use the stock MK4 0.4 mm nozzle, 0.20 mm Standard process and
PLA presets, with Arachne selected. Only `wall_sequence` changes between
projects: Inner/Outer/Inner (`mk4-ioi`), Inner/Outer (`mk4-io`), and
Outer/Inner (`mk4-oi`).

In `mk4-ioi-shared`, the first hole ends at X 0.7 instead of −0.83, leaving
1.4 mm between the holes. An inset-1 wall now touches two inset-0 walls.
Both holes, slots, and the connecting material are synthetic.
