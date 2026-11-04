# Arachne fuzzy F1 inputs

SPEC308583e3a, with F1 single-group whole coverage and F2 mask/split
separation. This folder contains input-only 3MFs, never Native output goldens.
Generate from repo root with `python3 tests/parity/arachne-fuzzy-f1/generate.py`.
ZIP timestamps/settings ordering and manifold edge/positive-volume checks are
deterministic; provenance.json binds source/container/mesh/settings hashes.

29 matrix inputs cover six noise variants × three modes, contour/hole/inset
types, first layer, zero/sub-threshold thickness, grouped Ripple phase, open
neck/varying width, short output padding and two inactive controls. Single
object/model volume and uniform fuzzy config give F1 whole coverage; there is
no F2 mask/split acceptance claim. The open-neck input reuses the already
checked C0-neck0.25 geometry/container; other meshes are newly authored inputs.
Generated prisms span Z=0..0.8mm and four0.2mm layers.

The two high-width Native rejection inputs are removed from this matrix.
Their raw preflight remains outside Git and is described in
`docs/architecture/arachne-fuzzy-f1-native-preflight.md`.

Tests call the validated byte API with explicit SlicerContext and the real CLI.
Seeded replay compares full ordered output for every matrix case; deterministic
Ripple/inactive cases also compare full ordered output in official mode.
Official active stochastic cases are explicitly diagnostic only and never
count as strict Native parity PASS. No coordinate normalization or chance retry.
The Context case additionally checks seed0/1 full replay, repeat0 after1 and
independent jobs; CLI default seed0 is covered by the main matrix.

Per-run fresh cache/root. Only the generic
slice producer is used; stable-options/dynamics oracles are not needed. Input, commands, producer hashes, Native raw output,
API/CLI status/output and strict-vs-diagnostic evidence stay outside Git.
