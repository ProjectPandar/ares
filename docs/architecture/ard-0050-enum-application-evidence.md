# ARD-0050: Preserve upstream enum wire values in parity evidence

`PrintConfig.cpp:279-285,2158-2179` and `PrintConfig.hpp::WallSequence`
own the three wall-order wire values, including two strings containing `/`.
Their Rust destination is the existing typed `ProcessWallSequence` and
`project_slice::perimeters::{classic,arachne}` ordering rewrite. This slice
repairs its CLI validation shell, not perimeter generation or a new pipeline.

The option sweep currently inserts the raw value label in an application
proof filename. The original accepts and exports the value, but the harness
interprets `/` as a directory separator and fails before comparison.

Keep option injection, human-readable labels, proof JSON and full ordered
G-code comparison unchanged. Use a fixed-length SHA-256 of the UTF-8 case
label for the filename component; keep the inventory-owned key prefix.
Both generic and typed-width proof writes must use the same case identity.
Identity must remain stable when a sweep filters or reorders cases. Do not
sanitize the actual value, create label-named directories or exempt failures.

Include all three enum values on the existing Ender-3 cube smoke context,
with effective-value proofs and retained full outputs. This does not certify
every wall geometry, firmware, printer or other enum domain. This context
has two wall loops; the three-wall Inner/Outer/Inner-specific ordering needs
a separate activated geometry regression, not a stronger claim here. Existing core
scaffolding remains a temporary shell around the cited upstream concepts;
adjacent perimeter ports and broader domain failures remain separate work.
