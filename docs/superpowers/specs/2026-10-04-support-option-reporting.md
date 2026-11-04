# Support Option evidence acceptance slice

Source/destination/included/deferred: ARD-0101. At the public 3MF
complete-output and CLI report seam, compare each support_type enum value.
The report must count all four strict comparisons, preserve real divergence
as FAIL with the first difference, and agree with subprocess failure status.
Paired case manifests remain unchanged, including repeated native retries.
This does not claim that a correctly reported divergent slice has parity.

Delete obsolete private unimplemented-support classifier assertions, retain
unrelated activation and all complete-output checks. Verify seeded and official
single-CPU reports, workspace Nextest and fmt/Clippy.
