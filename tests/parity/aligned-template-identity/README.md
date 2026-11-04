# AlignedRectilinear explicit-template identity inputs

Input-only3MFs copied byte-for-byte from the accepted read-only discovery.
No Native G-code, expected output snapshot or numerical Native oracle is stored.
Only frozen input SHA identities appear in the thin shared-check tests.

Source model/settings ancestor is
`tests/parity/fill-group-flow-ratio-key/equal-density.3mf`: one authored75-layer
overhang ModelPart and its existing same-density GUI modifier. All geometry,
transforms, placement and non-settings ZIP members are unchanged. The discovery
sets top/bottom/internal-solid patterns to alignedrectilinear and adds the
modifier's explicit solid rotation template; global solid direction stays45.

| Input | Global solid rotation template | Modifier template | Role |
|---|---|---|---|
|aligned-local45.3mf|empty|45|positive|
|aligned-global45-local45.3mf|45|45|sole-key global-template control|
|aligned-local90.3mf|empty|90|sole-key modifier-template control|

Relative to the positive, the first control changes ONLY
`Metadata/project_settings.config::solid_infill_rotate_template` empty→45.
The second changes ONLY the modifier's
`Metadata/model_settings.config::solid_infill_rotate_template`45→90.
No other settings/model data differ. Tests call existing
`native_stream_case::check` directly: fresh Native double replay, public
validated API and CLI, complete bytes under the existing generator/date-only
policy. No copied runner/normalizer or harness changes.

Input SHA256:
- aligned-local45:
  `779f7758a8ca330b2cbd2cf0688b1c7a154bbd44297f36c3af977c9f433956f9`
- aligned-global45-local45:
  `14987398e25c5ba9483311f74a2c86ce39f6ae5db236583831abb15cdcba3954`
- aligned-local90:
  `19ee9aa23dfe578d4542f8e785fc27f0048fbb2bd930eee783da70bd2b737630`

Upstream pin8500fcd: `Fill.cpp:937,941,1136` preserves template-nonempty identity;
comparator/equality288/316 retains it. `FillRectilinear.hpp:38-46` separately
returns zero class layer angle. GUI registration is `PrintConfig.cpp:4007-4016`
and `GUI/Tab.cpp:2493`; aligned patterns are listed2074-2113.

The accepted discovery's first strict positive failure involves **M73/progress
ordering**, not merely a later infill move. Group-identity evidence is external:
Native has false/true regional keys, while Ares forces aligned keys true and
merges them. No Native line/count/coordinate is asserted in tests or added to
these inputs. Future production scope is specified in
`docs/architecture/2026-10-21-aligned-template-identity-spec.md`; this RED does
not implement it. Ironing/concentric and shared helpers stay frozen.
