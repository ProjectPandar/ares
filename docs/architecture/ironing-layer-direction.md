# Object ironing: top-layer direction and alignment

**Upstream:** OrcaSlicer v2.5.0 `src/libslic3r/Fill/Fill.cpp:1796–1806`,
`Layer::make_ironing`. `:1899–1903` passes the narrowed angle and fixed flag
into the filler. `Fill/FillBase.cpp:454–458` uses the flag to skip alternation.

**Rust boundary:** `project_slice/fill_entities/ironing/{areas,spans}.rs`:
replace separate span-key and fill calculations with one `(f64 radians, bool)`
calculation. Reuse the effective region's `top_layer_direction` field.

- Use `top_layer_direction` as the base when it is nonnegative, bypassing
  template evaluation. Otherwise use the existing solid direction/template
  evaluation for the current layer.
- Explicit fixed ironing uses zero instead of that base. Add `ironing_angle`
  in radians, then the existing model `atan2(m10,m00)` offset when enabled.
- Keep the result in `f64` for span grouping and narrow once, at the filler.
  The fixed flag is true if explicit fixed ironing is enabled, the top-layer
  direction is set, or the solid rotation template is nonempty.

**Acceptance:** Use `orca_parity` with the same synthetic K1 3MF in both
engines (from an STL box, with an instance rotation for the alignment cases).
Compare exact, nonempty ordered ironing XY extrusion segments with Native:

- `top=23`, `template=11,61`: the top angle takes precedence (40°, not 78°).
- `top=37`, no template: the same base as the solid fallback, but the odd last
  layer must not alternate (54°, not 144°).
- Fixed 17°, aligned to a 31° rotation: preserve the 48° path to the emitted
  coordinate precision (X99.305, not X99.304).
- Controls: unset top direction; explicit fixed with top and template set;
  alignment disabled with literal fixed 48° on the same rotated instance.

Ordinary surface fill, rotation grammar, ZAA, pattern dispatch, span merge
policy and G-code emission are out of scope. The role-scoped comparison does
not change any existing whole-file parity check.
