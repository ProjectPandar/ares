# spiral-vase-object-start-z

RED fixtures for the spiral vase object-start Z-only line spec
(`docs/architecture/spiral-vase-object-start-z-spec.md`).

- `a1-single-zhop0.3mf` — the witness scene (`cube_with_hole.obj`, A1,
  `spiral_mode = 1`, by layer) with `z_hop = 0`; produced by editing the
  project settings JSON of the admission slice's `single-object-by-layer`
  fixture. SHA256
  `27501792b3793d29974f773defb264d1de8342c87ebf8d7cc2189ef4ee8cd935`.
  Diverges identically to the witness (byte 81702) — the missing-line
  class is hop-independent.

The witness case itself reuses the committed
`../spiral-vase-admission/single-object-by-layer.3mf` (same bytes; SHA256
`a8d0ed5f032f22a3b8968c9c648f621b913323abebaa417077f359b112b0ae38`) —
no duplicate copy here.
