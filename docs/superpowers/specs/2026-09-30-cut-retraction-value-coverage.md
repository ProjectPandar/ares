# Cut-retraction value coverage

Validate the `PrintConfig.cpp` cut-retraction definitions, generated
nullable filament overrides and `compute_filament_override_value`,
through the existing `GCode.cpp::_do_export` placeholder rewrite.
Rust destination: CLI option-domain generation/activation and public
project-to-complete-G-code tests; see ARD-0047.

Include 2 machine Boolean cases, 3 machine distance cases (10, 18,
seeded interior), 3 nullable filament Boolean cases and 4 nullable filament
distance cases. Nil is a distinct input state and must be verified as
applied, then observed falling back to the selected machine setting.
Add a reverse machine-Boolean nil witness so both fallback values are
checked: twelve generated cases and thirteen dedicated E2E comparisons.

Acceptance: failing domain/activation tests before repair; all thirteen
original-reference complete-output comparisons with runtime witnesses;
actual selected sweep; workspace Nextest, fmt/Clippy, Tier 1 compiler
checks, and independent six-axis repair/re-review before code/doc commit.
No byte-normalization changes or source-shape pinning tests.

Other nullable numeric options, mixed vectors, multi-tool physical cutting
and other printers remain deferred. Scaffolding remains a temporary
upstream rewrite validation shell, not an independent Ares feature.
