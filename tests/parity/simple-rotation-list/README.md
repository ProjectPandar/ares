# Simple rotation list input-only fixtures

Frozen byte copies of the five approved discovery inputs. No Native G-code,
expected numerical output or golden stream is stored here. The five thin
unconditional tests directly reuse the existing shared NativeCase lifecycle.

| Input | `sparse_infill_rotate_template` | SHA256 |
|---|---|---|
|space-list.3mf|`45 90`|703ea50de971fcfd681ff41e40aadc43e7c3cfab8db13a884da0c8e0a0e268af|
|tab-list.3mf|`45\t90`|ceeb9090d40c3bae21884e06e74994e69d0a694be11da05a7606ebc60b8c0396|
|comma-list.3mf|`45,90`|aa4bf45f18ef6c40f9e80953e7e7857f952a62e1c30fa78e8814c14c802b0799|
|single-angle.3mf|`45`|5ba6ae35c3f589214c84f51d32123861dc9666dd8f46cc8acf76b7406f0937d1|
|meta-space-control.3mf|`+45 +90`|1e23f416cc2deb8824981740a4a59cd3cb3dd50700b954b235cffbdb8d0d2220|

Source: actual Native-exported existing authored overhang input for
`infill_rotate_template::t24_sparse_list_control`; discovery provenance in
`${EVIDENCE_ROOT}/w3-simple-rotation-readonly-3fbad615e/README.md`.
Classic walls, one unpainted object, Rectilinear sparse/multiline1, monotonic
solid patterns. Common GUI settings disable brim/skirt. All geometry,
transforms and non-settings ZIP members remain exact to the same source.
Between these five inputs only the sparse rotation-template value differs.
No mesh/position/density search; existing test fixtures are unchanged.

`PrintConfig.cpp:3993-4004`, `GUI/Tab.cpp:2474` expose the advanced String field.
`Config.hpp:895-900` accepts ASCII whitespace and `GUI/Field.cpp:524-549`
retains the validated `45 90` string. The tooltip requests commas, but does not
reject this ordinary GUI input. Tab is an additional whitespace case.

Meta-space is a parser-boundary control; editing its literal spelling in GUI
may canonicalize separators via the other Field.cpp branch. It is not the
primary literal GUI-authored witness, and does not authorize meta changes.

Owning upstream: `Fill.cpp:206-210`, `Config.hpp:868-890`; meta branch at
`Fill.cpp:63-68` is distinct. Future boundary is only simple-list tokenization/
trim, with all shared numeric helpers and other behavior deferred. Tests keep
ordinary complete-output equality; targets are never inverted or filtered.

SPEC: `docs/architecture/2026-10-21-simple-rotation-list-tokenization-spec.md`.
