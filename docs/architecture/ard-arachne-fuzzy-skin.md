# ARD: Arachne fuzzy skin owns junctions and compatible-region effects

Status: accepted. SPEC 308583e3a with separate F1/F2 RED→GREEN slices.

Owner: `PerimeterGenerator.cpp::process_arachne:2095,2125` and
`traverse_extrusions:378–435`, with
`Feature/FuzzySkin/FuzzySkin.cpp::apply_fuzzy_skin(Arachne::ExtrusionLine*)`
685–814 and `fuzzy_extrusion_line`223–291/345–412.
Rust boundary: private `project_slice/perimeters/arachne/fuzzy_skin` modules,
existing materialize/traverse/config and necessary source-context views.

Non-negotiable decisions:

- Mutate the actual scaled position/width/perimeter-index junction sequence
  before clipping or variable-width conversion. Do not flatten it into the
  Classic polygon adapter or restore constant width afterward.
- Preserve all modes and noise dispatch exactly, including Ripple bypassing
  the random mode switch and the scaled-domain literal width floor.
- Group actual compatible members; full config equality and same-effect
  equality are distinct source contracts. Preserve masks, priority, source-index
  splits and non-fuzzy boundary junctions; no representative-first-region shortcut.
- Region normalization first follows PrintObject.cpp:3707–3708's threshold→None
  rule after all overrides; then eligibility follows source type/layer/inset/
  contour rules. Group flags control simplification/reversal. The initial SPEC
  incorrectly described the helper threshold as private; preflight corrected it.
- A task owns its explicit-seed Context continuously. Reuse verified RNG/noise
  primitives, preserve draw order and real slice_z, keep core platform-neutral.
- Strict parity and diagnostic stochastic admission are separate evidence.
  The seeded/official boundary is fixed; never disguise
  official stochastic projections as complete byte parity.
- Preserve every assertion not demonstrated broken. The closure decision permits
  F1 to remove the broken closing assertion, citing PerimeterGenerator.cpp:
  534–540, NDEBUG and the Native closure-width-audit.json in its commit message.
  Removing an adjacent-joint assertion requires demonstrated breakage; a
  necessary downstream width repair requires source evidence and a scope amendment.
  Retiring the one obsolete rejection test is a separate explicit commit.
- Keep each file <=400LOC; no new guards, fallback, skipped fuzzy geometry,
  probes, suppressions or committed Native outputs.

See [SPEC](arachne-fuzzy-skin-spec.md) for citations, included/deferred behavior,
public failure matrix and gates.
