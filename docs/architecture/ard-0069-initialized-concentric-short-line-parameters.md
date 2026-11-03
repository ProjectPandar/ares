# ARD-0069: Initialized concentric short-line parameters

Source boundary: `Fill/FillConcentric.cpp:92-100` and
`Fill/FillConcentricInternal.cpp:29-37` construct `Arachne::WallToolPathsParams`
without initializing `min_length_factor` or `is_top_or_bottom_layer`.
`Arachne/WallToolPaths.cpp::removeSmallLines` (:684-699) reads both fields.
Native official traces retain a subnormal factor and non-Boolean byte values
(`${TMP_ROOT}/ares-support-off-oracle-trace.log`, `${TMP_ROOT}/ares-official-small-confirm.log`).
These are confirmed undefined inputs, not intentional random noise.

The user permits a separate original test build that initializes confirmed
uninitialized fields. Keep the official binary and ARD-0068 seed-only build
unchanged. Create a third, explicitly identified seed-and-initialization build.
Only assign these two fields at the two fill construction sites; retain all
other geometry, clipping, traversal and RNG behavior.

Initialization convention: use factor 0.5 and top/bottom false, the explicit
common defaults in `WallToolPaths.cpp::make_paths_params` (:34-37,56).
This convention is source-cited but is not an existing initialized contract
of the defective fill constructors. Do not present it as untouched official
byte semantics. Record actual source patch, build identity and full outputs.

Rust destination: `project_slice/fill_entities/concentric/generate.rs`,
the `RawWallToolPathConfig` construction consumed by the existing Arachne
short-line filtering rewrite. If paired evidence confirms the convention,
replace the empirical standard/internal factor split with the initialized
common factor. No sampled-binary workaround, reference-specific switch or
new Ares slicing pipeline is allowed.

Public test seam: project bytes and explicit Context to full ordered G-code.
Use filled cube cases with fuzzy disabled/enabled to distinguish fill
selection from RNG consumption, and require a sensitive complete-output RED
before changing Rust. Retain official/seed-only controls and initialized
reference self-replay. Single-CPU reference scope follows ARD-0068; do not
claim general parallel replay or all printer/option coverage.

Include the two fill parameter initializations, their Rust consumer contract
and active full-output witnesses. Defer support generation, SLA application,
other uninitialized fields, multi-region fuzzy traversal and general original
parallel replay. Existing fixture harnesses remain temporary upstream
validation shells, not independent slicing behavior.

Reference prerequisite discovered during public KSR minimization:
`Print.hpp:582` declares `PrintObject::m_id` without an initializer;
`PrintObject.cpp:85-116` also leaves it unset. Ordinary object-label comments
(`GCode.cpp:5348-5351,5471-5475`) read it even when the conditional
`GCode::set_object_info` assignment (:8096) never runs. Same-input, same-seed,
single-CPU fresh processes produce different IDs. Independent read-only review
confirmed this is within the authorized initialization-only test-build scope.

Prepare another separate source/build identity with the retained seed/fill
patches plus `PrintObject::m_id{ 0 }`. Zero is an explicit test-build convention
for objects whose exclusion ID was never assigned, not untouched upstream
semantics; existing conditional sequential assignments remain unchanged.
The Rust destination is validation at the same project byte/G-code seam;
no new production object-ID implementation is introduced in this prerequisite.
This is an upstream `PrintObject` state repair in the original validation shell,
not an Ares pipeline feature. Preserve all previous builds and the
completed two-field reference scan. Prove same-input self-replay and complete paired
streams using the new identity before drawing further short-line conclusions.
Defer multi-object uniqueness and other M73/geometry differences; zeroing one
field neither establishes those properties nor closes the all-option goal.

Finite prerequisite commit: include only the reference initialization patches,
the complete native KSR self-replay test and its registration, and supporting
documents. Its exit is confirmed source/build identity, observed old-reference
RED/new-reference GREEN, final fmt/Clippy/workspace checks and independent
six-axis approval. The Rust consumer contract and sensitive public RED/GREEN
specified above remain open follow-up requirements, not this commit's claim.
