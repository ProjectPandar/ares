# Stars multiline input geometry

These are authored ASCII STL inputs, not Native-generated output. Both have
bounds X[-32,32], Y[-24,24], Z[0,4]. `prism.stl` is a closed 12-triangle box.
`holed-prism.stl` is a closed 32-triangle ring with an 8x8mm central through-hole;
outer walls face outward, hole walls inward, top upward and bottom downward.
Each planar quad is split on its first-to-third vertex diagonal.

Input SHA256:

- prism: `1d50f639d940e2a95c324b8f7ef1a33ba9bcf7e83706ba3cf0669f67f6feae7e`
- holed-prism: `6bfe629947754931ae6ddef2944a90ee0c61500c904bd65e2fdf5869d8d37301`

The CLI tests export and slice each runtime request through OrcaRunner, then
compare the complete ordered output using only generated-by normalization.
Native outputs, resolved projects and observation receipts remain outside git.
The 6mm hole neighborhood and 29/21mm exterior endpoint witnesses only establish
that sparse extrusion reaches both boundary classes; they do not weaken byte
comparison. See `docs/architecture/fill-stars-multiline-trapezoidal.md` for scope.
